import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist'
import pdfWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url'
import { createWorker, type Worker as TesseractWorker } from 'tesseract.js'

GlobalWorkerOptions.workerSrc = pdfWorkerUrl

const MAX_PDF_BYTES = 50 * 1024 * 1024
const MAX_TEXT_CHARS = 200_000
const OCR_RENDER_SCALE = 2
const MIN_EMBEDDED_TEXT_CHARS = 16

export interface PdfExtractProgress {
  phase: 'text' | 'ocr'
  pageNumber: number
  pageCount: number
  progress?: number
}

export interface PdfExtractResult {
  title: string
  text: string
  pageCount: number
  ocrPageCount: number
}

function normalizePageText(value: string) {
  return value.replace(/\s+/g, ' ').trim()
}

async function createOcrWorker(onProgress?: (progress: PdfExtractProgress) => void, pageNumber = 1, pageCount = 1) {
  return createWorker(['chi_sim', 'eng'], 1, {
    logger: (message) => {
      if (typeof message.progress === 'number') {
        onProgress?.({
          phase: 'ocr',
          pageNumber,
          pageCount,
          progress: Math.max(0, Math.min(1, message.progress))
        })
      }
    }
  })
}

async function recognizePdfPage(
  page: Awaited<ReturnType<Awaited<ReturnType<typeof getDocument>['promise']>['getPage']>>,
  worker: TesseractWorker
) {
  const viewport = page.getViewport({ scale: OCR_RENDER_SCALE })
  const canvas = document.createElement('canvas')
  canvas.width = Math.ceil(viewport.width)
  canvas.height = Math.ceil(viewport.height)
  const context = canvas.getContext('2d', { alpha: false })
  if (!context) throw new Error('无法创建 PDF OCR 画布')

  await page.render({ canvasContext: context, viewport, canvas }).promise
  const result = await worker.recognize(canvas)
  canvas.width = 1
  canvas.height = 1
  return normalizePageText(result.data.text || '')
}

export async function extractPdf(
  file: File,
  onProgress?: (progress: PdfExtractProgress) => void
): Promise<PdfExtractResult> {
  if (file.size > MAX_PDF_BYTES) {
    throw new Error('PDF 超过 50 MB 上限')
  }

  const data = new Uint8Array(await file.arrayBuffer())
  const pdf = await getDocument({ data }).promise
  const pageCount = pdf.numPages
  const pages: string[] = []
  let total = 0
  let ocrPageCount = 0
  let ocrWorker: TesseractWorker | null = null

  try {
    for (let pageNumber = 1; pageNumber <= pageCount; pageNumber += 1) {
      onProgress?.({ phase: 'text', pageNumber, pageCount })
      const page = await pdf.getPage(pageNumber)
      const content = await page.getTextContent()
      let pageText = normalizePageText(content.items
        .map((item) => ('str' in item ? item.str : ''))
        .join(' '))

      if (pageText.length < MIN_EMBEDDED_TEXT_CHARS) {
        if (!ocrWorker) {
          ocrWorker = await createOcrWorker(onProgress, pageNumber, pageCount)
        }
        onProgress?.({ phase: 'ocr', pageNumber, pageCount, progress: 0 })
        const ocrText = await recognizePdfPage(page, ocrWorker)
        if (ocrText.length > pageText.length) {
          pageText = ocrText
          ocrPageCount += 1
        }
      }

      if (!pageText) continue
      const pageLabel = `【PDF 第 ${pageNumber} 页】\n`
      const remaining = MAX_TEXT_CHARS - total - pageLabel.length
      if (remaining <= 0) break

      const clipped = pageText.length > remaining ? pageText.slice(0, remaining) : pageText
      pages.push(pageLabel + clipped)
      total += pageLabel.length + clipped.length + 2
    }
  } finally {
    if (ocrWorker) await ocrWorker.terminate().catch(() => undefined)
  }

  const text = pages.join('\n\n').trim()
  if (!text) {
    throw new Error('没有提取到可学习的文本；OCR 也未识别出有效内容')
  }

  let title = file.name.replace(/\.pdf$/i, '')
  try {
    const metadata = await pdf.getMetadata()
    const info = metadata.info as unknown as Record<string, unknown>
    const metadataTitle = typeof info.Title === 'string' ? info.Title.trim() : ''
    if (metadataTitle) title = metadataTitle
  } catch {
    // PDF metadata is optional. Filename remains the fallback title.
  }

  return { title, text, pageCount, ocrPageCount }
}
