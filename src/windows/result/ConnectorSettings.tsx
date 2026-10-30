import { Plus, Save, Trash2 } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { AppConfig, ContentConnectorProfile } from '../../domain/settings'
import { connectorApiKeyConfigured, saveConnectorApiKey } from '../../services/discovery-service'
import { replaceAppConfig } from '../../services/settings-service'

interface ConnectorSettingsProps {
  config: AppConfig
  onConfigChange: (config: AppConfig) => void
}

const PAID = [
  { id: 'brave', name: 'Brave Search', hint: 'X-Subscription-Token' },
  { id: 'tavily', name: 'Tavily', hint: 'API Key' },
  { id: 'exa', name: 'Exa', hint: 'x-api-key' }
] as const

function blankConnector(): ContentConnectorProfile {
  return {
    id: `custom-${crypto.randomUUID().slice(0, 8)}`,
    name: '自定义来源',
    endpoint: 'https://example.com/search',
    queryParam: 'q',
    itemsPath: 'results',
    titlePath: 'title',
    urlPath: 'url',
    snippetPath: 'snippet',
    contentPath: 'content',
    enabled: true
  }
}

export function ConnectorSettings({ config, onConfigChange }: ConnectorSettingsProps) {
  const [keys, setKeys] = useState<Record<string, string>>({})
  const [configured, setConfigured] = useState<Record<string, boolean>>({})
  const [selectedId, setSelectedId] = useState(config.connectors?.[0]?.id || '')
  const [draft, setDraft] = useState<ContentConnectorProfile | null>(config.connectors?.[0] || null)
  const [customKey, setCustomKey] = useState('')
  const [status, setStatus] = useState<string | null>(null)

  useEffect(() => {
    void Promise.all(PAID.map(async (item) => [item.id, await connectorApiKeyConfigured(item.id).catch(() => false)] as const))
      .then((items) => setConfigured(Object.fromEntries(items)))
  }, [])

  useEffect(() => {
    const item = config.connectors?.find((connector) => connector.id === selectedId) || null
    setDraft(item ? { ...item } : null)
    setCustomKey('')
  }, [config.connectors, selectedId])

  const savePaid = async (id: string) => {
    await saveConnectorApiKey(id, keys[id] || '')
    setConfigured((current) => ({ ...current, [id]: Boolean((keys[id] || '').trim()) }))
    setKeys((current) => ({ ...current, [id]: '' }))
    setStatus('已保存到 Windows 凭据管理器')
  }

  const saveCustom = async () => {
    if (!draft) return
    const existing = config.connectors || []
    const nextConnectors = existing.some((item) => item.id === selectedId)
      ? existing.map((item) => item.id === selectedId ? draft : item)
      : [...existing, draft]
    const next = await replaceAppConfig({ ...config, connectors: nextConnectors })
    if (draft.apiKeyHeader?.trim() && customKey.trim()) {
      await saveConnectorApiKey(draft.id, customKey)
    }
    onConfigChange(next)
    setSelectedId(draft.id)
    setStatus('Connector 已保存')
  }

  const removeCustom = async () => {
    if (!draft) return
    const next = await replaceAppConfig({
      ...config,
      connectors: (config.connectors || []).filter((item) => item.id !== draft.id)
    })
    onConfigChange(next)
    const first = next.connectors[0]?.id || ''
    setSelectedId(first)
    setDraft(next.connectors[0] || null)
    setStatus('Connector 已删除')
  }

  return (
    <div className="provider-settings">
      <section className="settings-card">
        <div className="settings-card-head">
          <div><h3>付费搜索</h3><p>API Key 只进 Windows 凭据管理器，不写配置文件。</p></div>
        </div>
        <div className="settings-grid provider-grid">
          {PAID.map((item) => (
            <label className="wide" key={item.id}>
              <span>{item.name} {configured[item.id] ? '· 已配置' : '· 未配置'}</span>
              <div className="provider-model-row">
                <input
                  type="password"
                  value={keys[item.id] || ''}
                  placeholder={configured[item.id] ? '输入新 Key 可覆盖' : item.hint}
                  onChange={(event) => setKeys((current) => ({ ...current, [item.id]: event.target.value }))}
                />
                <button type="button" disabled={!keys[item.id]?.trim()} onClick={() => void savePaid(item.id)}>
                  <Save size={14} />保存
                </button>
              </div>
            </label>
          ))}
        </div>
      </section>

      <section className="settings-card">
        <div className="settings-card-head">
          <div>
            <h3>自定义 HTTP / JSON Connector</h3>
            <p>适合有公开 GET 搜索接口的服务；用点路径指定结果数组与字段。</p>
          </div>
          <button type="button" onClick={() => {
            const next = blankConnector()
            setSelectedId('')
            setDraft(next)
            setStatus(null)
          }}><Plus size={14} />新建</button>
        </div>

        {(config.connectors?.length || 0) > 0 && (
          <label className="settings-field">
            <span>已有 Connector</span>
            <select value={selectedId} onChange={(event) => setSelectedId(event.target.value)}>
              <option value="">新建中的 Connector</option>
              {config.connectors.map((item) => <option key={item.id} value={item.id}>{item.name}</option>)}
            </select>
          </label>
        )}

        {draft && (
          <div className="settings-grid provider-grid">
            <label><span>ID</span><input value={draft.id} disabled={Boolean(selectedId)} onChange={(event) => setDraft({ ...draft, id: event.target.value })} /></label>
            <label><span>名称</span><input value={draft.name} onChange={(event) => setDraft({ ...draft, name: event.target.value })} /></label>
            <label className="wide"><span>Endpoint</span><input value={draft.endpoint} onChange={(event) => setDraft({ ...draft, endpoint: event.target.value })} /></label>
            <label><span>查询参数</span><input value={draft.queryParam} onChange={(event) => setDraft({ ...draft, queryParam: event.target.value })} /></label>
            <label><span>结果数组路径</span><input value={draft.itemsPath} onChange={(event) => setDraft({ ...draft, itemsPath: event.target.value })} /></label>
            <label><span>标题路径</span><input value={draft.titlePath} onChange={(event) => setDraft({ ...draft, titlePath: event.target.value })} /></label>
            <label><span>URL 路径</span><input value={draft.urlPath} onChange={(event) => setDraft({ ...draft, urlPath: event.target.value })} /></label>
            <label><span>摘要路径</span><input value={draft.snippetPath} onChange={(event) => setDraft({ ...draft, snippetPath: event.target.value })} /></label>
            <label><span>正文路径</span><input value={draft.contentPath} onChange={(event) => setDraft({ ...draft, contentPath: event.target.value })} /></label>
            <label><span>API Key Header（可选）</span><input value={draft.apiKeyHeader || ''} placeholder="Authorization" onChange={(event) => setDraft({ ...draft, apiKeyHeader: event.target.value || undefined })} /></label>
            {draft.apiKeyHeader && (
              <label><span>API Key</span><input type="password" value={customKey} placeholder="只保存到凭据管理器" onChange={(event) => setCustomKey(event.target.value)} /></label>
            )}
            <label className="settings-checkbox-row"><input type="checkbox" checked={draft.enabled} onChange={(event) => setDraft({ ...draft, enabled: event.target.checked })} /><span>启用</span></label>
            <div className="wide provider-actions">
              <button type="button" className="primary" onClick={() => void saveCustom()}><Save size={14} />保存 Connector</button>
              {selectedId && <button type="button" onClick={() => void removeCustom()}><Trash2 size={14} />删除</button>}
            </div>
          </div>
        )}
        {status && <p className="settings-hint">{status}</p>}
      </section>
    </div>
  )
}
