import { useState, useEffect } from 'react'
import { ArrowLeft, Plus, Play, Trash2, MessageSquare } from 'lucide-react'
import { listSessions, getSession, deleteSession, createSession, switchSession, type SessionMeta } from '../api'

interface Props {
  onNavigate?: (page: 'chat') => void
  onSessionChange?: () => void
}

export default function SessionsPage({ onNavigate, onSessionChange }: Props) {
  const [sessions, setSessions] = useState<SessionMeta[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedSession, setSelectedSession] = useState<{ id: string; title: string; messages: { role: string; content: string }[] } | null>(null)

  const loadSessions = async () => {
    setLoading(true)
    try {
      const resp = await listSessions()
      if (resp.success && resp.data) {
        setSessions(resp.data)
      }
    } catch (err) {
      console.error('Failed to load sessions', err)
    }
    setLoading(false)
  }

  useEffect(() => {
    loadSessions()
  }, [])

  const handleClickSession = async (id: string) => {
    try {
      const resp = await getSession(id)
      if (resp.success && resp.data) {
        setSelectedSession(resp.data)
      }
    } catch (err) {
      console.error('Failed to load session', err)
    }
  }

  const handleDelete = async (id: string) => {
    try {
      await deleteSession(id)
      setSessions((prev) => prev.filter((s) => s.id !== id))
      if (selectedSession?.id === id) {
        setSelectedSession(null)
      }
    } catch (err) {
      console.error('Failed to delete session', err)
    }
  }

  const handleUse = async (id: string) => {
    try {
      const resp = await switchSession(id)
      if (resp.success) {
        onSessionChange?.()
        onNavigate?.('chat')
      }
    } catch (err) {
      console.error('Failed to switch session', err)
    }
  }

  const handleNewSession = async () => {
    try {
      const resp = await createSession()
      if (resp.success && resp.data) {
        onSessionChange?.()
        onNavigate?.('chat')
      }
    } catch (err) {
      console.error('Failed to create session', err)
    }
  }

  if (selectedSession) {
    return (
      <>
        <div className="page-header">
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
            <button
              className="icon-btn"
              onClick={() => setSelectedSession(null)}
            >
              <ArrowLeft size={16} className="icon-btn-icon" />
              Back
            </button>
            <h2>{selectedSession.title}</h2>
          </div>
        </div>
        <div className="page-body">
          {selectedSession.messages.length === 0 ? (
            <div className="empty-state">
              <MessageSquare size={40} className="empty-state-icon" />
              <p>No messages in this session.</p>
            </div>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              {selectedSession.messages.map((msg, i) => (
                <div key={i} className={`message ${msg.role}`}>
                  {msg.content}
                </div>
              ))}
            </div>
          )}
        </div>
      </>
    )
  }

  return (
    <>
      <div className="page-header">
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <h2>Sessions ({sessions.length})</h2>
          <button className="send-btn btn-sm" onClick={handleNewSession}>
            <Plus size={14} />
            New Session
          </button>
        </div>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading sessions...
          </div>
        ) : sessions.length === 0 ? (
          <div className="empty-state">
            <MessageSquare size={40} className="empty-state-icon" />
            <p>No sessions yet. Start a chat!</p>
          </div>
        ) : (
          <div className="card">
            {sessions.map((session) => (
              <div key={session.id} className="session-item">
                <div className="session-info" onClick={() => handleClickSession(session.id)}>
                  <div className="session-title">{session.title}</div>
                  <div className="session-meta">
                    {session.message_count} messages &middot; {new Date(session.created_at * 1000).toLocaleString()}
                    {session.agent_id !== 'default' && <span className="session-agent-badge">{session.agent_id}</span>}
                  </div>
                </div>
                <div style={{ display: 'flex', gap: '6px' }}>
                  <button className="session-use" onClick={() => handleUse(session.id)}>
                    <Play size={12} />
                    Use
                  </button>
                  <button className="session-delete" onClick={() => handleDelete(session.id)}>
                    <Trash2 size={12} />
                    Delete
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  )
}
