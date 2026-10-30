import { RotateCcw, Save } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import type { AppConfig, ModelRef, TaskRoute } from '../../domain/settings'
import { getAiUsageSummary, type AiUsageSummary } from '../../services/ai-usage-service'
import { replaceAppConfig } from '../../services/settings-service'

interface KnowledgeAiSettingsProps {
  config: AppConfig
  onConfigChange: (config: AppConfig) => void
}

interface TaskDraft {
  providerId: string
  model: string
  fallbackProviderId: string
  fallbackModel: string
}

const TASKS = [
  { id: 'internalize', label: '提炼' },
  { id: 'question_generation', label: '出题' },
  { id: 'assessment', label: '判分' },
  { id: 'review_plan', label: '复习规划' },
  { id: 'research', label: '研究' }
] as const

function routeDraft(route?: TaskRoute): TaskDraft {
  const fallback = route?.fallbacks?.[0]
  return {
    providerId: route?.primary.providerId || '',
    model: route?.primary.model || '',
    fallbackProviderId: fallback?.providerId || '',
    fallbackModel: fallback?.model || ''
  }
}

function formatTokens(value: number) {
  return new Intl.NumberFormat('zh-CN').format(value)
}

export function KnowledgeAiSettings({ config, onConfigChange }: KnowledgeAiSettingsProps) {
  const enabledProviders = useMemo(() => config.providers.filter((provider) => provider.enabled), [config.providers])
  const [drafts, setDrafts] = useState<Record<string, TaskDraft>>({})
  const [usage, setUsage] = useState<AiUsageSummary | null>(null)
  const [saving, setSaving] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  useEffect(() => {
    const next: Record<string, TaskDraft> = {}
    for (const task of TASKS) next[task.id] = routeDraft(config.taskRoutes?.[task.id])
    setDrafts(next)
  }, [config.taskRoutes])

  useEffect(() => {
    void getAiUsageSummary(30).then(setUsage).catch((usageError) => setError(String(usageError)))
  }, [])

  const updateDraft = (taskId: string, patch: Partial<TaskDraft>) => {
    setDrafts((current) => ({ ...current, [taskId]: { ...current[taskId], ...patch } }))
  }

  const saveTask = async (taskId: string) => {
    const draft = drafts[taskId]
    if (!draft?.providerId || !draft.model.trim()) return
    const fallbacks: ModelRef[] = draft.fallbackProviderId && draft.fallbackModel.trim()
      ? [{ providerId: draft.fallbackProviderId, model: draft.fallbackModel.trim() }]
      : []
    const route: TaskRoute = {
      primary: { providerId: draft.providerId, model: draft.model.trim() },
      fallbacks,
      options: config.taskRoutes?.[taskId]?.options || {}
    }
    setSaving(taskId)
    setError(null)
    try {
      const next = await replaceAppConfig({
        ...config,
        taskRoutes: { ...(config.taskRoutes || {}), [taskId]: route }
      })
      onConfigChange(next)
    } catch (saveError) {
      setError(String(saveError))
    } finally {
      setSaving(null)
    }
  }

  const clearTask = async (taskId: string) => {
    const taskRoutes = { ...(config.taskRoutes || {}) }
    delete taskRoutes[taskId]
    setSaving(taskId)
    setError(null)
    try {
      onConfigChange(await replaceAppConfig({ ...config, taskRoutes }))
    } catch (clearError) {
      setError(String(clearError))
    } finally {
      setSaving(null)
    }
  }

  return (
    <div className="action-routes-editor">
      <div className="action-routes-head">
        <span>未单独配置的任务会继续使用“内化”动作的模型与 Fallback。</span>
      </div>
      {error && <div className="settings-error">{error}</div>}
      <div className="action-route-list">
        {TASKS.map((task) => {
          const draft = drafts[task.id] || routeDraft()
          const configured = Boolean(config.taskRoutes?.[task.id])
          return (
            <div className="action-route-item" key={task.id}>
              <div className="action-route-row">
                <div className="action-route-name"><span>{task.label}</span></div>
                <select value={draft.providerId} onChange={(event) => updateDraft(task.id, {
                  providerId: event.target.value,
                  model: config.providers.find((provider) => provider.id === event.target.value)?.defaultModel || ''
                })}>
                  <option value="">跟随内化路由</option>
                  {enabledProviders.map((provider) => <option key={provider.id} value={provider.id}>{provider.name}</option>)}
                </select>
                <input value={draft.model} disabled={!draft.providerId} placeholder="模型" onChange={(event) => updateDraft(task.id, { model: event.target.value })} />
                <div className="action-route-controls">
                  <button type="button" title="保存任务路由" disabled={!draft.providerId || !draft.model.trim() || saving === task.id} onClick={() => void saveTask(task.id)}>
                    <Save size={13} strokeWidth={1.8} />
                  </button>
                  <button type="button" title="恢复跟随内化" disabled={!configured || saving === task.id} onClick={() => void clearTask(task.id)}>
                    <RotateCcw size={13} strokeWidth={1.8} />
                  </button>
                </div>
              </div>
              {draft.providerId && (
                <div className="action-route-fallbacks">
                  <div className="action-route-fallback">
                    <span>备用</span>
                    <select value={draft.fallbackProviderId} onChange={(event) => updateDraft(task.id, {
                      fallbackProviderId: event.target.value,
                      fallbackModel: config.providers.find((provider) => provider.id === event.target.value)?.defaultModel || ''
                    })}>
                      <option value="">不设置</option>
                      {enabledProviders.map((provider) => <option key={provider.id} value={provider.id}>{provider.name}</option>)}
                    </select>
                    <input value={draft.fallbackModel} disabled={!draft.fallbackProviderId} placeholder="备用模型" onChange={(event) => updateDraft(task.id, { fallbackModel: event.target.value })} />
                  </div>
                </div>
              )}
            </div>
          )
        })}
      </div>

      <div className="action-routes-head">
        <span>最近 30 天用量</span>
        <strong>{usage ? `${formatTokens(usage.inputTokens + usage.outputTokens)} tokens · ${usage.requests} 次请求` : '读取中…'}</strong>
      </div>
      {usage && usage.items.length > 0 && (
        <div className="action-route-list">
          {usage.items.map((item) => (
            <div className="action-route-item" key={`${item.task}:${item.providerId}:${item.model}`}>
              <div className="action-route-row">
                <div className="action-route-name"><span>{TASKS.find((task) => task.id === item.task)?.label || item.task}</span></div>
                <span>{item.model}</span>
                <span>{formatTokens(item.inputTokens)} in / {formatTokens(item.outputTokens)} out</span>
                <small>{item.cachedTokens > 0 ? `${formatTokens(item.cachedTokens)} cached · ` : ''}{item.requests} 次</small>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  )
}
