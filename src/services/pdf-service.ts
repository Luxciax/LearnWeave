import { GlobalWorkerOptions, getDocument } from 'pdfjs-dist'
import pdfWorkerUrl from 'pdfjs-dist/build/pdf.worker.min.mjs?url'

GlobalWorkerOptions.workerSrc = pdfWorkerUrl

const MAX_PDF_BYTES = 50 * 1024 * 1024
const MAX_TEXT_CHARS = 200_000

export interface PdfExtractResult {
  title: string
  text: string
  pageCount: number
}

export async function extractPdf(file: File): Promise<PdfExtractResult> {
  if (file.size > MAX_PDF_BYTES) {
    throw new Error('PDF 超过 50 MB 上限')
  }

  const data = new Uint8Array(await file.arrayBuffer())
  const pdf = await getDocument({ data }).promise
  const pageCount = pdf.numPages
  const pages: string[] = []
  let total = 0

  for (let pageNumber = 1; pageNumber <= pageCount; pageNumber += 1) {
    const page = await pdf.getPage(pageNumber)
    const content = await page.getTextContent()
    const pageText = content.items
      .map((item) => ('str' in item ? item.str : ''))
      .join(' ')
      .replace(/\s+/g, ' ')
      .trim()

    if (!pageText) continue
    const remaining = MAX_TEXT_CHARS - total
    if (remaining <= 0) break

    const clipped = pageText.length > remaining ? pageText.slice(0, remaining) : pageText
    pages.push(clipped)
    total += clipped.length + 2
  }

  const text = pages.join('\n\n').trim()
  if (!text) {
    throw new Error('没有提取到可学习的文本；扫描版 PDF 目前需要先做 OCR')
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

  return { title, text, pageCount }
}
