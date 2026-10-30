import { listen } from '@tauri-apps/api/event'
import { Activity, BookOpen, FileText, Inbox as InboxIcon, Library, Plus, RotateCcw, Search, Settings2, Sparkles, Tags, Telescope, Waypoints } from 'lucide-react'
import { useEffect, useState } from 'react'
import { getCurrentReviewSession } from '../../services/knowledge-service'
import { CaptureDialog } from './CaptureDialog'
import { DiscoverView } from './DiscoverView'
import { InboxView } from './InboxView'
import { KnowledgeGraphView } from './KnowledgeGraphView'
import { KnowledgeHome } from './KnowledgeHome'
import { KnowledgeLibraryView } from './KnowledgeLibraryView'
import { LibrarianView } from './LibrarianView'
import { ReviewCenter } from './ReviewCenter'
import { SourceLibraryView } from './SourceLibraryView'
import { TaskCenterView } from './TaskCenterView'
import { TodayView } from './TodayView'
import { TopicExpansionView } from './TopicExpansionView'
import { TopicsView } from './TopicsView'
import './knowledge-workspace.css'

type WorkspaceView = 'search' | 'expand' | 'today' | 'inbox' | 'tasks' | 'organize' | 'library' | 'sources' | 'topics' | 'graph' | 'review'

interface KnowledgeWorkspaceProps {
  onOpenSettings: () => void
  searchRequest?: { query: string; nonce: number } | null
  onSearchRequestConsumed?: () => void
  captureRequestNonce?: number
}

const NAV_ITEMS: Array<{ id: WorkspaceView; label: string; icon: typeof Search }> = [
  { id: 'today', label: '今天', icon: BookOpen },
  { id: 'inbox', label: '收件箱', icon: InboxIcon },
  { id: 'library', label: '知识库', icon: Library },
  { id: 'search', label: '发现', icon: Search },
  { id: 'topics', label: '主题', icon: Tags },
  { id: 'review', label: '复习', icon: RotateCcw },
  { id: 'sources', label: '来源', icon: FileText },
  { id: 'tasks', label: 'AI 任务', icon: Activity }
]

export function KnowledgeWorkspace({ onOpenSettings, searchRequest = null, onSearchRequestConsumed, captureRequestNonce = 0 }: KnowledgeWorkspaceProps) {
  const [view, setView] = useState<WorkspaceView>('today')
  const [selectedKnowledgeId, setSelectedKnowledgeId] = useState<string | null>(null)
  const [selectedQuestionId, setSelectedQuestionId] = useState<string | null>(null)
  const [libraryTopicId, setLibraryTopicId] = useState<string | null>(null)
  const [selectedSourceId, setSelectedSourceId] = useState<string | null>(null)
  const [sourceBackView, setSourceBackView] = useState<'expand' | null>(null)
  const [graphCenterId, setGraphCenterId] = useState<string | null>(null)
  const [selectedInboxId, setSelectedInboxId] = useState<string | null>(null)
  const [researchTarget, setResearchTarget] = useState<{ planId: string | null; gapId: string | null }>({ planId: null, gapId: null })
  const [searchSeed, setSearchSeed] = useState('')
  const [captureOpen, setCaptureOpen] = useState(false)

  useEffect(() => {
    if (captureRequestNonce > 0) setCaptureOpen(true)
  }, [captureRequestNonce])

  useEffect(() => {
    if (!searchRequest) return
    setSelectedKnowledgeId(null)
    setSelectedQuestionId(null)
    setLibraryTopicId(null)
    setSelectedSourceId(null)
    setGraphCenterId(null)
    setSearchSeed(searchRequest.query)
    setView('search')
    onSearchRequestConsumed?.()
  }, [searchRequest?.nonce, onSearchRequestConsumed])

  useEffect(() => {
    let disposed = false
    const cleanup: Array<() => void> = []
    void Promise.all([
      listen('app://home', () => {
        if (disposed) return
        setSelectedKnowledgeId(null)
        setSelectedQuestionId(null)
        setLibraryTopicId(null)
        setSelectedSourceId(null)
        setGraphCenterId(null)
        setSearchSeed('')
        setView('today')
      })
    ]).then((unlisteners) => {
      if (disposed) unlisteners.forEach((unlisten) => unlisten())
      else cleanup.push(...unlisteners)
    })
    return () => {
      disposed = true
      cleanup.forEach((unlisten) => unlisten())
    }
  }, [])

  const openKnowledge = (knowledgeUnitId: string, questionId: string | null = null) => {
    setSelectedKnowledgeId(knowledgeUnitId)
    setSelectedQuestionId(questionId)
  }

  const openInbox = (inboxId: string | null = null) => {
    setSelectedKnowledgeId(null)
    setSelectedQuestionId(null)
    setSelectedInboxId(inboxId)
    setView('inbox')
  }

  const openSourceFromResearch = (sourceId: string) => {
    setSelectedKnowledgeId(null)
    setSelectedQuestionId(null)
    setSelectedSourceId(sourceId)
    setSourceBackView('expand')
    setView('sources')
  }

  const openResearch = (planId: string | null = null, gapId: string | null = null) => {
    setSelectedKnowledgeId(null)
    setSelectedQuestionId(null)
    setResearchTarget({ planId, gapId })
    setView('expand')
  }

  const continueReviewSession = async () => {
    try {
      const session = await getCurrentReviewSession()
      const next = session?.items.find((item) => item.status === 'pending')
      if (next) {
        openKnowledge(next.knowledgeUnitId, next.questionId)
      } else {
        setSelectedKnowledgeId(null)
        setSelectedQuestionId(null)
      }
    } catch (error) {
      console.error('Failed to continue review session', error)
      setSelectedKnowledgeId(null)
      setSelectedQuestionId(null)
    }
  }

  const changeView = (next: WorkspaceView) => {
    setSelectedKnowledgeId(null)
    setSelectedQuestionId(null)
    if (next !== 'library') setLibraryTopicId(null)
    if (next !== 'sources') setSelectedSourceId(null)
    setSourceBackView(null)
    if (next !== 'graph') setGraphCenterId(null)
    setSelectedInboxId(null)
    setResearchTarget({ planId: null, gapId: null })
    if (next !== 'search') setSearchSeed('')
    setView(next)
  }

  const content = (() => {
    if (selectedKnowledgeId) {
      return (
        <KnowledgeHome
          onOpenSettings={onOpenSettings}
          listenForAppNavigation={false}
          initialKnowledgeId={selectedKnowledgeId}
          initialQuestionId={selectedQuestionId}
          detailBackLabel={view === 'expand' ? '主题扩充' : view === 'inbox' ? '收件箱' : view === 'tasks' ? 'AI 任务' : view === 'organize' ? '整理建议' : view === 'library' ? '知识库' : view === 'sources' ? '来源' : view === 'topics' ? '主题' : view === 'graph' ? '知识地图' : view === 'review' ? '学习与复习' : '今天'}
          onDetailClose={() => {
            setSelectedKnowledgeId(null)
            setSelectedQuestionId(null)
          }}
          onReviewContinue={view === 'review' ? () => void continueReviewSession() : undefined}
          onOpenGraph={(knowledgeUnitId) => {
            setSelectedKnowledgeId(null)
            setSelectedQuestionId(null)
            setGraphCenterId(knowledgeUnitId)
            setView('graph')
          }}
        />
      )
    }

    if (view === 'search') {
      return (
        <DiscoverView
          initialQuery={searchSeed}
          onBack={() => {
            setSearchSeed('')
            setView('today')
          }}
          onCaptured={() => undefined}
          onOpenCapture={() => setCaptureOpen(true)}
          onOpenSettings={onOpenSettings}
        />
      )
    }
    if (view === 'expand') {
      return (
        <TopicExpansionView
          initialPlanId={researchTarget.planId}
          initialGapId={researchTarget.gapId}
          onOpenKnowledge={openKnowledge}
          onOpenSource={openSourceFromResearch}
          onOpenSettings={onOpenSettings}
        />
      )
    }
    if (view === 'today') {
      return (
        <TodayView
          onOpenInbox={openInbox}
          onOpenCapture={() => setCaptureOpen(true)}
          onOpenSearch={() => changeView('search')}
          onOpenSettings={onOpenSettings}
          onOpenReview={() => changeView('review')}
          onOpenResearch={openResearch}
          onOpenKnowledge={openKnowledge}
          onOpenSuggestions={() => changeView('organize')}
          onOpenTasks={() => changeView('tasks')}
        />
      )
    }
    if (view === 'inbox') {
      return <InboxView initialInboxId={selectedInboxId} onOpenKnowledge={openKnowledge} />
    }
    if (view === 'tasks') {
      return (
        <TaskCenterView
          onOpenKnowledge={openKnowledge}
          onOpenInbox={openInbox}
          onOpenSuggestions={() => changeView('organize')}
        />
      )
    }
    if (view === 'organize') {
      return (
        <LibrarianView
          backLabel="今天"
          onBack={() => changeView('today')}
          onChanged={() => undefined}
          onOpenKnowledge={openKnowledge}
        />
      )
    }
    if (view === 'library') {
      return (
        <KnowledgeLibraryView
          initialTopicId={libraryTopicId}
          onOpenKnowledge={openKnowledge}
        />
      )
    }
    if (view === 'sources') {
      return (
        <SourceLibraryView
          initialSourceId={selectedSourceId}
          onSourceChange={(sourceId) => {
            setSelectedSourceId(sourceId)
            if (!sourceId && sourceBackView) {
              setView(sourceBackView)
              setSourceBackView(null)
            }
          }}
          onOpenKnowledge={openKnowledge}
        />
      )
    }
    if (view === 'topics') {
      return (
        <TopicsView
          onOpenTopic={(topicId) => {
            setLibraryTopicId(topicId)
            setView('library')
          }}
        />
      )
    }
    if (view === 'graph') {
      return (
        <KnowledgeGraphView
          initialCenterId={graphCenterId}
          onCenterChange={setGraphCenterId}
          onOpenKnowledge={openKnowledge}
        />
      )
    }
    if (view === 'review') {
      return <ReviewCenter onOpenKnowledge={openKnowledge} />
    }
    return <KnowledgeHome onOpenSettings={onOpenSettings} />
  })()

  return (
    <section className="knowledge-workspace">
      <aside className="knowledge-sidebar" aria-label="LearnWeave 导航">
        <div className="knowledge-sidebar-brand">
          <BookOpen size={17} strokeWidth={1.8} />
          <strong>LearnWeave</strong>
        </div>
        <button className="knowledge-capture-button" type="button" onClick={() => setCaptureOpen(true)}>
          <Plus size={16} strokeWidth={1.9} />
          <span>捕获</span>
        </button>
        <nav>
          {NAV_ITEMS.map((item) => {
            const Icon = item.icon
            return (
              <button
                type="button"
                key={item.id}
                className={view === item.id && !selectedKnowledgeId ? 'active' : undefined}
                onClick={() => changeView(item.id)}>
                <Icon size={16} strokeWidth={1.8} />
                <span>{item.label}</span>
              </button>
            )
          })}
        </nav>
        <div className="knowledge-sidebar-footer">
          <button type="button" onClick={onOpenSettings}>
            <Settings2 size={16} strokeWidth={1.8} />
            <span>设置</span>
          </button>
        </div>
      </aside>
      <div className="knowledge-workspace-content">{content}</div>
      {captureOpen && (
        <CaptureDialog
          onClose={() => setCaptureOpen(false)}
          onCaptured={() => {
            setCaptureOpen(false)
            openInbox()
          }}
        />
      )}
    </section>
  )
}
