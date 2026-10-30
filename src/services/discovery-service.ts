import { invoke } from '@tauri-apps/api/core'

export interface DiscoverConnectorDescriptor {
  id: string
  label: string
  kind: 'public' | 'feed' | 'paid' | 'custom'
  configured: boolean
  requiresKey: boolean
  queryHint: string
}

export interface DiscoverResult {
  connectorId: string
  title: string
  url?: string
  author?: string
  snippet: string
  contentText: string
}

export function listDiscoverConnectors() {
  return invoke<DiscoverConnectorDescriptor[]>('discover_connectors')
}

export function searchDiscoverConnector(connectorId: string, query: string, limit = 10) {
  return invoke<DiscoverResult[]>('discover_search', { connectorId, query, limit })
}

export function connectorApiKeyConfigured(connectorId: string) {
  return invoke<boolean>('connector_api_key_configured', { connectorId })
}

export function saveConnectorApiKey(connectorId: string, apiKey: string) {
  return invoke<void>('connector_save_api_key', { connectorId, apiKey })
}
