import { invoke } from '@tauri-apps/api/core'
import { emit } from '@tauri-apps/api/event'
import type { ZhihuSearchResult } from '../types'

export function isZhihuAccessSecretConfigured() {
  return invoke<boolean>('zhihu_access_secret_configured')
}

export async function saveZhihuAccessSecret(accessSecret: string) {
  await invoke<void>('zhihu_save_access_secret', { accessSecret })
  void emit('discover://sources-changed', { connectorId: 'zhihu' }).catch(() => undefined)
}

export function searchZhihu(query: string, count = 10) {
  return invoke<ZhihuSearchResult>('zhihu_search', { query, count })
}

export function searchZhihuExpanded(query: string, count = 20) {
  return invoke<ZhihuSearchResult>('zhihu_global_search', { query, count })
}

export function openZhihuSearch(query: string) {
  return invoke<void>('zhihu_open_search', { query })
}
