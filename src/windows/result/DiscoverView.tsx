import { listen } from '@tauri-apps/api/event'
import { LoaderCircle, Plus, Search, Settings2 } from 'lucide-react'
import { useCallback, useEffect, useState } from 'react'
import { SEARCH_CONNECTORS, zhihuConnector } from '../../connectors/search-connectors'
import { ZhihuSearchView } from './ZhihuSearchView'

interface DiscoverViewProps {
  initialQuery: string
  onBack: () => void
  onCaptured: () => void
  onOpenCapture: () => void
  onOpenSettings: () => void
}

export function DiscoverView({
  initialQuery,
  onBack,
  onCaptured,
  onOpenCapture,
  onOpenSettings
}: DiscoverViewProps) {
  const [configuredConnectorIds, setConfiguredConnectorIds] = useState<string[] | null>(null)

  const refreshSources = useCallback(async () => {
    const checks = await Promise.all(SEARCH_CONNECTORS.map(async (connector) => {
      try {
        return await connector.isConfigured() ? connector.id : null
      } catch {
        return null
      }
    }))
    setConfiguredConnectorIds(checks.filter((id): id is string => Boolean(id)))
  }, [])

  useEffect(() => {
    let disposed = false
    let unlisten: (() => void) | undefined
    void refreshSources()
    void listen('discover://sources-changed', () => {
      if (!disposed) void refreshSources()
    }).then((cleanup) => {
      if (disposed) cleanup()
      else unlisten = cleanup
    })
    return () => {
      disposed = true
      unlisten?.()
    }
  }, [refreshSources])

  if (configuredConnectorIds === null) {
    return (
      <section className="discover-empty" aria-label="发现">
        <LoaderCircle size={18} strokeWidth={1.8} className="spin" />
        <span>正在检查可用的搜索来源…</span>
      </section>
    )
  }

  if (configuredConnectorIds.length === 0) {
    return (
      <section className="discover-empty" aria-label="发现">
        <Search size={22} strokeWidth={1.6} />
        <div>
          <h1>还没有可搜索的来源</h1>
          <p>发现页只负责从外部来源找内容。现在可以配置搜索来源，或者直接捕获已有的文本和网页。</p>
        </div>
        <div className="discover-empty-actions">
          <button type="button" className="primary" onClick={onOpenCapture}>
            <Plus size={14} strokeWidth={1.8} />捕获内容
          </button>
          <button type="button" onClick={onOpenSettings}>
            <Settings2 size={14} strokeWidth={1.8} />配置搜索来源
          </button>
        </div>
      </section>
    )
  }

  if (configuredConnectorIds.includes(zhihuConnector.id)) {
    return (
      <ZhihuSearchView
        initialQuery={initialQuery}
        onBack={onBack}
        onCaptured={onCaptured}
        onOpenSettings={onOpenSettings}
      />
    )
  }

  return (
    <section className="discover-empty" aria-label="发现">
      <Search size={22} strokeWidth={1.6} />
      <div>
        <h1>搜索来源已连接</h1>
        <p>当前版本还没有这个 Connector 的结果视图。</p>
      </div>
    </section>
  )
}
