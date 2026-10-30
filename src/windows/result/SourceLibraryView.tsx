import { AlertTriangle, ArrowLeft, ExternalLink, Search } from 'lucide-react'
import { type FormEvent, useCallback, useEffect, useMemo, useState } from 'react'
import { getSourceDetail, listSources, openSourceUrl } from '../../services/knowledge-service'
import type { KnowledgeStatus, SourceDetailRecord, SourceKind, SourceLibraryItem, SourcePlatform, SourceRecord } from '../../types'

interface SourceLibraryViewProps {
  initialSourceId?: string | null
  onSourceChange: (sourceId: string | null) => void
  onOpenKnowledge: (knowledgeUnitId: string) => void
}

const PLATFORM_LABELS: Record<SourcePlatform, string> = {
  zhihu: '知乎',
  web: '网页',
  pdf: 'PDF',
  desktop: '桌面',
  manual: '手动',
  unknown: '未知来源'
}

const SOURCE_KIND_LABELS: Record<SourceKind, string> = {
  web: '网页',
  desktop: '桌面',
  document: '文档',
  manual: '手动',
  unknown: '未知来源'
}

const CONNECTOR_LABELS: Record<string, string> = {
  zhihu: '知乎',
  'web-url': '网页 URL'
}

const STATUS_LABELS: Record<KnowledgeStatus, string> = {
  captured: '待提炼',
  processed: '已处理',
  weak: '薄弱',
  learning: '学习中',
  reviewing: '复习中',
  mastered: '已掌握'
}

const SOURCE_KIND_OPTIONS: Array<{ value: '' | SourceKind; label: string }> = [
  { value: '', label: '全部类型' },
  { value: 'web', label: '网页' },
  { value: 'desktop', label: '桌面' },
  { value: 'document', label: '文档' },
  { value: 'manual', label: '手动' },
  { value: 'unknown', label: '未知来源' }
]

const CONNECTOR_OPTIONS = [
  { value: '', label: '全部 Connector' },
  { value: 'zhihu', label: '知乎' },
  { value: 'web-url', label: '网页 URL' }
] as const

function sourceLabel(item: Pick<SourceLibraryItem, 'sourceKind' | 'connectorId' | 'platform'> | SourceRecord) {
  return item.connectorId ? (CONNECTOR_LABELS[item.connectorId] || item.connectorId) : (SOURCE_KIND_LABELS[item.sourceKind] || PLATFORM_LABELS[item.platform])
}

function sourceTitle(item: SourceLibraryItem) {
  return item.title?.trim() || item.author?.trim() || `${sourceLabel(item)}来源`
}

function sourceExcerpt(value: string) {
  const text = value.replace(/\s+/g, ' ').trim()
  return text.length > 180 ? `${text.slice(0, 180)}…` : text
}

function zhihuDuplicateKey(item: SourceLibraryItem) {
  if ((item.connectorId !== 'zhihu' && item.platform !== 'zhihu') || !item.url) return null
  const canonicalUrl = item.url.split('#')[0].split('?')[0].replace(/\/+$/, '')
  const normalizedText = item.selectedText.replace(/\s+/g, ' ').trim()
  return canonicalUrl && normalizedText ? `${canonicalUrl}\u001f${normalizedText}` : null
}

export function SourceLibraryView({ initialSourceId = null, onSourceChange, onOpenKnowledge }: SourceLibraryViewProps) {
  const [items, setItems] = useState<SourceLibraryItem[]>([])
  const [detail, setDetail] = useState<SourceDetailRecord | null>(null)
  const [queryDraft, setQueryDraft] = useState('')
  const [query, setQuery] = useState('')
  const [sourceKind, setSourceKind] = useState<'' | SourceKind>('')
  const [connectorId, setConnectorId] = useState('')
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)

  const duplicateSummary = useMemo(() => {
    const groups = new Map<string, string[]>()
    for (const item of items) {
      const key = zhihuDuplicateKey(item)
      if (!key) continue
      const ids = groups.get(key) ?? []
      ids.push(item.id)
      groups.set(key, ids)
    }
    const duplicateGroups = [...groups.values()].filter((ids) => ids.length > 1)
    return {
      groupCount: duplicateGroups.length,
      sourceIds: new Set(duplicateGroups.flat())
    }
  }, [items])

  const refreshList = useCallback(async () => {
    setLoading(true)
    try {
      setItems(await listSources({
        query: query || undefined,
        sourceKind: sourceKind || undefined,
        connectorId: connectorId || undefined,
        limit: 200
      }))
      setError(null)
    } catch (loadError) {
      setError(String(loadError))
    } finally {
      setLoading(false)
    }
  }, [connectorId, query, sourceKind])

  const loadDetail = useCallback(async (sourceId: string) => {
    setLoading(true)
    try {
      setDetail(await getSourceDetail(sourceId))
      setError(null)
    } catch (loadError) {
      setDetail(null)
      setError(String(loadError))
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    if (initialSourceId) void loadDetail(initialSourceId)
    else {
      setDetail(null)
      void refreshList()
    }
  }, [initialSourceId, loadDetail, refreshList])

  const submitSearch = (event: FormEvent) => {
    event.preventDefault()
    setQuery(queryDraft.trim())
  }

  const openSource = (sourceId: string) => {
    onSourceChange(sourceId)
  }

  const closeSource = () => {
    setDetail(null)
    onSourceChange(null)
  }

  if (initialSourceId) {
    return (
      <section className="sources-view source-detail-view" aria-label="来源详情">
        <header className="source-detail-toolbar">
          <button type="button" onClick={closeSource}><ArrowLeft size={15} strokeWidth={1.8} />来源</button>
          {detail?.source.url && (
            <button type="button" onClick={() => void openSourceUrl(detail.source.url!)}>
              <ExternalLink size={13} strokeWidth={1.8} />打开原始链接
            </button>
          )}
        </header>
        {error ? (
          <button type="button" className="workspace-error" onClick={() => void loadDetail(initialSourceId)}>读取失败，点击重试</button>
        ) : loading || !detail ? (
          <div className="workspace-empty">正在读取来源…</div>
        ) : (
          <div className="source-detail-content">
            <header className="source-detail-heading">
              <span>{sourceLabel(detail.source)}</span>
              <h1>{detail.source.title || detail.source.author || '未命名来源'}</h1>
              <p>
                {detail.source.author ? `${detail.source.author} · ` : ''}
                {new Date(detail.source.capturedAt).toLocaleDateString()} · 已内化 {detail.knowledgeUnits.length} 条知识
              </p>
            </header>

            <section className="source-original-section">
              <div className="source-section-heading"><strong>原文</strong><span>Source</span></div>
              {detail.source.contextBefore && <p className="source-context">{detail.source.contextBefore}</p>}
              <blockquote>{detail.source.selectedText}</blockquote>
              {detail.source.contextAfter && <p className="source-context">{detail.source.contextAfter}</p>}
            </section>

            <section className="source-derived-section">
              <div className="source-section-heading"><strong>内化知识</strong><span>{detail.knowledgeUnits.length} 条</span></div>
              <div className="source-derived-list">
                {detail.knowledgeUnits.map((unit, index) => (
                  <button type="button" key={unit.id} onClick={() => onOpenKnowledge(unit.id)}>
                    <span className="source-derived-order">{String(index + 1).padStart(2, '0')}</span>
                    <span className="source-derived-copy">
                      <strong>{unit.coreClaim.trim() || '待提炼知识'}</strong>
                      <small>
                        {STATUS_LABELS[unit.status]}
                        {unit.reviewCount > 0 ? ` · 掌握 ${unit.masteryScore}` : ' · 待检验'}
                        {unit.archivedAt !== null ? ' · 已归档' : ''}
                      </small>
                    </span>
                    <b>查看</b>
                  </button>
                ))}
              </div>
            </section>
          </div>
        )}
      </section>
    )
  }

  return (
    <section className="sources-view" aria-label="来源">
      <header className="workspace-page-heading">
        <div>
          <h1>来源</h1>
          <p>{loading ? '正在读取…' : `${items.length} 个原始来源`}</p>
        </div>
      </header>

      <form className="library-search" onSubmit={submitSearch}>
        <Search size={17} strokeWidth={1.8} />
        <input
          value={queryDraft}
          placeholder="搜索标题、作者、链接或原文…"
          onChange={(event) => setQueryDraft(event.target.value)}
        />
        {queryDraft && <button type="submit">搜索</button>}
      </form>

      <div className="source-filter-row">
        <label>
          <span>类型</span>
          <select value={sourceKind} onChange={(event) => setSourceKind(event.target.value as '' | SourceKind)}>
            {SOURCE_KIND_OPTIONS.map((option) => <option key={option.value || 'all'} value={option.value}>{option.label}</option>)}
          </select>
        </label>
        <label>
          <span>Connector</span>
          <select value={connectorId} onChange={(event) => setConnectorId(event.target.value)}>
            {CONNECTOR_OPTIONS.map((option) => <option key={option.value || 'all'} value={option.value}>{option.label}</option>)}
          </select>
        </label>
      </div>

      {duplicateSummary.groupCount > 0 && (
        <div className="source-duplicate-notice">
          <AlertTriangle size={14} strokeWidth={1.8} />
          <span>当前列表发现 {duplicateSummary.groupCount} 组历史重复来源：知乎链接和正文完全一致。这里只做诊断，不会自动合并旧数据。</span>
        </div>
      )}

      {error ? (
        <button type="button" className="workspace-error" onClick={() => void refreshList()}>读取失败，点击重试</button>
      ) : loading ? (
        <div className="workspace-empty">正在读取来源…</div>
      ) : items.length === 0 ? (
        <div className="workspace-empty">没有符合当前条件的来源。</div>
      ) : (
        <div className="source-list">
          {items.map((item) => {
            const duplicate = duplicateSummary.sourceIds.has(item.id)
            return (
            <button type="button" className={`source-item${duplicate ? ' duplicate-suspect' : ''}`} key={item.id} onClick={() => openSource(item.id)}>
              <div className="source-item-main">
                <div className="source-item-meta">
                  <span>{sourceLabel(item)}</span>
                  {item.author && <strong>{item.author}</strong>}
                  <small>{new Date(item.capturedAt).toLocaleDateString()}</small>
                  {duplicate && <em>历史重复</em>}
                </div>
                <h3>{sourceTitle(item)}</h3>
                <p>{sourceExcerpt(item.selectedText)}</p>
              </div>
              <div className="source-item-count"><b>{item.knowledgeCount}</b><span>条知识</span></div>
            </button>
            )
          })}
        </div>
      )}
    </section>
  )
}
