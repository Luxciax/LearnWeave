import { emit } from '@tauri-apps/api/event'
import { Check, FileText, Link2, LoaderCircle, Type, X } from 'lucide-react'
import { useEffect, useRef, useState } from 'react'
import { captureKnowledge, fetchWebUrl, internalizeKnowledge } from '../../services/knowledge-service'
import { extractPdf } from '../../services/pdf-service'
import type { SourceCaptureInput } from '../../types'

interface CaptureDialogProps {
  onClose: () => void
  onCaptured: () => void
}

type CaptureState = 'idle' | 'saving' | 'saved' | 'error'
type CaptureMode = 'text' | 'url' | 'pdf'

export function CaptureDialog({ onClose, onCaptured }: CaptureDialogProps) {
  const [mode, setMode] = useState<CaptureMode>('text')
  const [text, setText] = useState('')
  const [url, setUrl] = useState('')
  const [pdfFile, setPdfFile] = useState<File | null>(null)
  const [pdfText, setPdfText] = useState('')
  const [pdfTitle, setPdfTitle] = useState('')
  const [pdfPages, setPdfPages] = useState(0)
  const [state, setState] = useState<CaptureState>('idle')
  const [error, setError] = useState('')
  const textareaRef = useRef<HTMLTextAreaElement | null>(null)
  const urlRef = useRef<HTMLInputElement | null>(null)

  useEffect(() => {
    if (mode === 'text') textareaRef.current?.focus()
    else if (mode === 'url') urlRef.current?.focus()
  }, [mode])

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && state !== 'saving') onClose()
    }
    window.addEventListener('keydown', onKeyDown)
    return () => window.removeEventListener('keydown', onKeyDown)
  }, [onClose, state])

  const setCaptureMode = (next: CaptureMode) => {
    if (state === 'saving' || state === 'saved') return
    setMode(next)
    setState('idle')
    setError('')
  }

  const selectPdf = async (file: File | null) => {
    setPdfFile(file)
    setPdfText('')
    setPdfTitle('')
    setPdfPages(0)
    setError('')
    if (!file) {
      setState('idle')
      return
    }

    setState('saving')
    try {
      const result = await extractPdf(file)
      setPdfText(result.text)
      setPdfTitle(result.title)
      setPdfPages(result.pageCount)
      setState('idle')
    } catch (pdfError) {
      setState('error')
      setError(String(pdfError))
    }
  }

  const submit = async () => {
    if (state === 'saving') return
    const value = mode === 'text'
      ? text.trim()
      : mode === 'url'
        ? url.trim()
        : pdfText.trim()
    if (!value) return

    setState('saving')
    setError('')
    try {
      let input: SourceCaptureInput
      if (mode === 'url') {
        const fetched = await fetchWebUrl(value)
        input = {
          platform: 'web',
          sourceKind: 'web',
          connectorId: 'web-url',
          selectedText: fetched.text,
          url: fetched.url,
          title: fetched.title
        }
      } else if (mode === 'pdf') {
        input = {
          platform: 'pdf',
          sourceKind: 'document',
          connectorId: 'pdf-upload',
          selectedText: value,
          title: pdfTitle || pdfFile?.name
        }
      } else {
        input = {
          platform: 'manual',
          sourceKind: 'manual',
          selectedText: value
        }
      }

      const captured = await captureKnowledge(input)
      void emit('knowledge://changed', { knowledgeUnitId: captured.knowledgeUnit.id }).catch(() => undefined)
      try {
        await internalizeKnowledge(captured.knowledgeUnit.id)
      } catch (jobError) {
        console.error('Capture was saved but AI internalization could not be scheduled', jobError)
      }
      setState('saved')
      window.setTimeout(() => onCaptured(), 260)
    } catch (captureError) {
      setState('error')
      setError(String(captureError))
    }
  }

  const canSubmit = (mode === 'text'
    ? text.trim()
    : mode === 'url'
      ? url.trim()
      : pdfText.trim()).length > 0

  return (
    <div className="capture-dialog-backdrop" role="presentation" onMouseDown={(event) => {
      if (event.target === event.currentTarget && state !== 'saving') onClose()
    }}>
      <section className="capture-dialog" role="dialog" aria-modal="true" aria-labelledby="capture-dialog-title">
        <header>
          <div>
            <strong id="capture-dialog-title">捕获内容</strong>
            <span>把想学习的内容送进收件箱</span>
          </div>
          <button type="button" aria-label="关闭" title="关闭" disabled={state === 'saving'} onClick={onClose}>
            <X size={16} strokeWidth={1.8} />
          </button>
        </header>

        <div className="capture-dialog-tabs" role="tablist" aria-label="捕获方式">
          <button className={mode === 'text' ? 'active' : undefined} type="button" role="tab" aria-selected={mode === 'text'} onClick={() => setCaptureMode('text')}>
            <Type size={14} strokeWidth={1.8} />
            <span>文本</span>
          </button>
          <button className={mode === 'url' ? 'active' : undefined} type="button" role="tab" aria-selected={mode === 'url'} onClick={() => setCaptureMode('url')}>
            <Link2 size={14} strokeWidth={1.8} />
            <span>URL</span>
          </button>
          <button className={mode === 'pdf' ? 'active' : undefined} type="button" role="tab" aria-selected={mode === 'pdf'} onClick={() => setCaptureMode('pdf')}>
            <FileText size={14} strokeWidth={1.8} />
            <span>PDF</span>
          </button>
        </div>

        {mode === 'text' ? (
          <label className="capture-dialog-field">
            <span>内容</span>
            <textarea
              ref={textareaRef}
              value={text}
              maxLength={200000}
              placeholder="粘贴一段值得保存、理解或复习的内容…"
              disabled={state === 'saving' || state === 'saved'}
              onChange={(event) => {
                setText(event.target.value)
                if (state === 'error') {
                  setState('idle')
                  setError('')
                }
              }}
            />
          </label>
        ) : mode === 'url' ? (
          <label className="capture-dialog-field capture-dialog-url-field">
            <span>网页地址</span>
            <input
              ref={urlRef}
              type="url"
              value={url}
              placeholder="https://example.com/article"
              disabled={state === 'saving' || state === 'saved'}
              onChange={(event) => {
                setUrl(event.target.value)
                if (state === 'error') {
                  setState('idle')
                  setError('')
                }
              }}
              onKeyDown={(event) => {
                if (event.key === 'Enter') {
                  event.preventDefault()
                  void submit()
                }
              }}
            />
            <small>会读取公开网页正文；本机、局域网和非网页内容不会访问。</small>
          </label>
        ) : (
          <label className="capture-dialog-field">
            <span>PDF 文件</span>
            <input
              type="file"
              accept="application/pdf,.pdf"
              disabled={state === 'saving' || state === 'saved'}
              onChange={(event) => void selectPdf(event.target.files?.[0] || null)}
            />
            <small>
              {pdfText
                ? `已提取 ${pdfPages} 页 · ${pdfText.length.toLocaleString()} 字符`
                : '本地解析 PDF，不上传原文件；扫描版 PDF 暂不做 OCR。'}
            </small>
          </label>
        )}

        {state === 'error' && <p className="capture-dialog-error">捕获失败：{error}</p>}

        <footer>
          <small>
            {mode === 'text'
              ? `${text.trim().length.toLocaleString()} 字符`
              : mode === 'url'
                ? '网页内容会保留最终 URL 与标题'
                : pdfFile?.name || '选择 PDF 文件'}
          </small>
          <button
            className="capture-dialog-submit"
            type="button"
            disabled={!canSubmit || state === 'saving' || state === 'saved'}
            onClick={() => void submit()}>
            {state === 'saving' ? <LoaderCircle size={15} className="spin" /> : state === 'saved' ? <Check size={15} /> : null}
            <span>
              {state === 'saving'
                ? mode === 'url'
                  ? '正在读取网页'
                  : mode === 'pdf'
                    ? pdfText
                      ? '正在保存'
                      : '正在解析 PDF'
                    : '正在保存'
                : state === 'saved'
                  ? '已进入收件箱'
                  : '加入收件箱'}
            </span>
          </button>
        </footer>
      </section>
    </div>
  )
}
