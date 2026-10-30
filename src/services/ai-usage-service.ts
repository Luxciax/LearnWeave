import { invoke } from '@tauri-apps/api/core'

export interface AiUsageItem {
  task: string
  providerId: string
  model: string
  inputTokens: number
  outputTokens: number
  cachedTokens: number
  requests: number
}

export interface AiUsageSummary {
  inputTokens: number
  outputTokens: number
  cachedTokens: number
  requests: number
  items: AiUsageItem[]
}

export function getAiUsageSummary(days = 30) {
  return invoke<AiUsageSummary>('ai_usage_summary', { days })
}
