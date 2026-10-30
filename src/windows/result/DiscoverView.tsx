import { ExternalLink, LoaderCircle, Plus, Search, Settings2 } from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import {
  listDiscoverConnectors,
  searchDiscoverConnector,
  type DiscoverConnectorDescriptor,
  type DiscoverResult
} from '../../services/discovery-service'
import { captureKnowledge, internalizeKnowledge, openSourceUrl } from '../../services/knowledge-service'
import { isZhihuAccessSecretConfigured } from '../../services/zhihu-service'
import { ZhihuSearchView } from './ZhihuSearchView'

interface DiscoverViewProps {
  initialQuery: string
  onBack: () => void
  onCaptured: () => void
  onOpenCapture: () => void
  onOpenSettings: () => void
}

const ZHIHU: DiscoverConnectorDescriptor = {
  id: 'zhihu',
  label: '知乎',
  kind: 'public',
  configured: true,
  requiresKey: true,
  queryHint: '搜索知乎'
}

export function DiscoverView({
  initialQuery,
  onBack,
  onCaptured,
  onOpenCapture,
  onOpenSettings
}: DiscoverViewProps) {
  const [connectors, setConnectors] = useState<DiscoverConnectorDescriptor[] | null>(null)
  const [connectorId, setConnectorId] = useState('')
  const [query, setQuery] = useState(initialQuery)
  const [results, setResults] = useState<DiscoverResult[]>([])
  const [loading, setLoading] = useState(false)
  const [capturingUrl, setCapturingUrl] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  const refreshSources = useCallback(async () => {
    const [items, zhihuConfigured] = await Promise.all([
      listDiscoverConnectors(),
      isZhihuAccessSecretConfigured().catch(() => false)
    ])
    const next = zhihuConfigured ? [ZHIHU, ...items] : items
    setConnectors(next)
    setConnectorId((current) => current && next.some((item) => item.id === current)
      ? current
      : next.find((item) => item.configured)?.id || next[0]?.id || '')
  }, [])

  useEffect(() => {
    void refreshSources().catch((loadError) => {
      setError(String(loadError))
      setConnectors([])
    })
  }, [refreshSources])

  const selected = useMemo(
    () => connectors?.find((item) => item.id === connectorId) || null,
    [connectorId, connectors]
  )

  const search = useCallback(async () => {
    if (!connectorId || connectorId === 'zhihu' || !query.trim()) return
    setLoading(true)
    setError(null)
    try {
      setResults(await searchDiscoverConnector(connectorId, query.trim(), 12))
    } catch (searchError) {
      setResults([])
      setError(String(searchError))
    } finally {
      setLoading(false)
    }
  }, [connectorId, query])

  useEffect(() => {
    if (!initialQuery || !connectorId || connectorId === 'zhihu') return
    void search()
  }, [connectorId])

  const capture = async (item: DiscoverResult) => {
    const text = (item.contentText || item.snippet || item.title).trim()
    if (!text) return
    setCapturingUrl(item.url || item.title)
    setError(null)
    try {
      const captured = await captureKnowledge({
        platform: 'web',
        sourceKind: 'web',
        connectorId: item.connectorId,
        selectedText: text,
        url: item.url,
        title: item.title,
        author: item.author
      })
      await internalizeKnowledge(captured.knowledgeUnit.id).catch(() => undefined)
      onCaptured()
    } catch (captureError) {
      setError(String(captureError))
    } finally {
      setCapturingUrl(null)
    }
  }

  if (connectors === null) {
    return (
      <section className="discover-empty" aria-label="发现">
        <LoaderCircle size={18} strokeWidth={1.8} className="spin" />
        <span>正在检查可用来源…</span>
      </section>
    )
  }

  if (selected?.id === 'zhihu') {
    return (
      <section className="discover-shell">
        <div className="discover-source-bar">
          <select value={connectorId} onChange={(event) => setConnectorId(event.target.value)}>
            {connectors.map((item) => (
              <option key={item.id} value={item.id} disabled={!item.configured}>
                {item.label}{!item.configured ? '（未配置）' : ''}
              </option>
            ))}
          </select>
          <span>知乎保留原生搜索与阅读器</span>
        </div>
        <ZhihuSearchView
          initialQuery={initialQuery}
          onBack={onBack}
          onCaptured={onCaptured}
          onOpenSettings={onOpenSettings}
        />
      </section>
    )
  }

  return (
    <section className="discover-shell" aria-label="发现">
      <header className="discover-generic-head">
        <div>
          <h1>发现</h1>
          <p>从外部来源找内容，确认值得学再送进收件箱。</p>
        </div>
        <div className="discover-empty-actions">
          <button type="button" onClick={onOpenCapture}><Plus size={14} />捕获已有内容</button>
          <button type="button" onClick={onOpenSettings}><Settings2 size={14} />来源设置</button>
        </div>
      </header>

      <div className="discover-search-row">
        <select value={connectorId} onChange={(event) => {
          setConnectorId(event.target.value)
          setResults([])
          setError(null)
        }}>
          {connectors.map((item) => (
            <option key={item.id} value={item.id} disabled={!item.configured}>
              {item.label}{!item.configured ? '（未配置 Key）' : ''}
            </option>
          ))}
        </select>
        <input
          value={query}
          placeholder={selected?.queryHint || '输入查询'}
          onChange={(event) => setQuery(event.target.value)}
          onKeyDown={(event) => {
            if (event.key === 'Enter') void search()
          }}
        />
        <button type="button" className="primary" disabled={!selected?.configured || loading || !query.trim()} onClick={() => void search()}>
          {loading ? <LoaderCircle size={15} className="spin" /> : <Search size={15} />}
          搜索
        </button>
      </div>

      {error && <div className="settings-error">{error}</div>}

      <div className="discover-result-list">
        {results.map((item, index) => {
          const captureKey = item.url || item.title
          return (
            <article className="discover-result-card" key={captureKey + index}>
              <div className="discover-result-copy">
                <strong>{item.title}</strong>
                {item.author && <small>{item.author}</small>}
                <p>{item.snippet || item.contentText || '这个结果只有来源链接，可打开原文后再决定是否捕获。'}</p>
              </div>
              <div className="discover-result-actions">
                {item.url && (
                  <button type="button" onClick={() => void openSourceUrl(item.url!)}>
                    <ExternalLink size={14} />原文
                  </button>
                )}
                <button type="button" className="primary" disabled={capturingUrl === captureKey} onClick={() => void capture(item)}>
                  {capturingUrl === captureKey ? <LoaderCircle size={14} className="spin" /> : <Plus size={14} />}
                  加入收件箱
                </button>
              </div>
            </article>
          )
        })}
        {!loading && results.length === 0 && (
          <div className="discover-empty compact">
            <Search size={20} />
            <span>{selected?.kind === 'feed' ? '粘贴一个 RSS / Atom 地址开始读取。' : '输入关键词开始发现。'}</span>
          </div>
        )}
      </div>
    </section>
  )
}
