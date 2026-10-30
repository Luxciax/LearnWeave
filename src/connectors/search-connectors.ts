import {
  isZhihuAccessSecretConfigured,
  searchZhihu,
  searchZhihuExpanded
} from '../services/zhihu-service'
import type { ZhihuSearchResult } from '../types'

export interface SearchConnectorDescriptor {
  id: string
  label: string
  isConfigured: () => Promise<boolean>
}

export interface SearchConnector<TResult> extends SearchConnectorDescriptor {
  search: (query: string, count?: number) => Promise<TResult>
  expandSearch?: (query: string, count?: number) => Promise<TResult>
}

export const zhihuConnector: SearchConnector<ZhihuSearchResult> = {
  id: 'zhihu',
  label: '知乎',
  isConfigured: isZhihuAccessSecretConfigured,
  search: searchZhihu,
  expandSearch: searchZhihuExpanded
}

export const SEARCH_CONNECTORS: readonly SearchConnectorDescriptor[] = [zhihuConnector]
