import { useState, useEffect, useCallback } from 'react'
import { getCurrentSession, createSession, listSessions, switchSession } from '../api/sessions'
import type { ChatMessage, ToolCallMsg } from '../api/types'

function deserializeMessages(raw: any[]): ChatMessage[] {
  const msgs: ChatMessage[] = []
  let pendingToolCalls: ToolCallMsg[] = []
  for (const m of raw) {
    if (m.role === 'user') {
      pendingToolCalls = []
      msgs.push({ role: 'user', content: m.content || '' })
    } else if (m.role === 'assistant') {
      msgs.push({
        role: 'assistant',
        content: m.content || '',
        reasoning: m.reasoning || undefined,
        toolCalls: pendingToolCalls.length > 0 ? [...pendingToolCalls] : undefined,
      })
      pendingToolCalls = []
    } else if (m.role === 'tool_call') {
      pendingToolCalls.push({
        name: m.name || '',
        args: m.args || '',
        result: m.result || '',
        step: 0,
        total_steps: 1,
      })
    } else if (m.role === 'image') {
      pendingToolCalls = []
      msgs.push({
        role: 'image',
        content: '',
        image: {
          path: m.path || '',
          alt_text: m.alt_text || '',
          width: m.width || 0,
          height: m.height || 0,
          format: m.format || '',
          url: m.url || `/api/images/${m.path || ''}`,
        },
      })
    } else if (m.role === 'evaluation') {
      pendingToolCalls = []
      msgs.push({
        role: 'evaluation',
        content: m.content || '',
        evaluation: {
          tool: m.tool || '',
          valid: m.valid ?? true,
          issues: m.issues || [],
        },
      })
    } else if (m.role === 'quality') {
      msgs.push({
        role: 'quality',
        content: m.content || '',
        quality: {
          score: m.score || '0',
          complete: m.complete ?? true,
          issues: m.issues || [],
          references_valid: m.references_valid ?? false,
        },
      })
    } else if (m.role === 'feedback') {
      pendingToolCalls = []
      msgs.push({
        role: 'feedback',
        content: m.content || '',
        feedback: {
          positive: m.positive ?? true,
          message: m.message || '',
        },
      })
    }
  }
  return msgs
}

interface SessionState {
  messages: ChatMessage[]
  setMessages: React.Dispatch<React.SetStateAction<ChatMessage[]>>
  sessionId: string | null
  setSessionId: React.Dispatch<React.SetStateAction<string | null>>
  sessionTitle: string
  sessionAgent: string | null
  loading: boolean
  reload: () => void
  newChat: () => Promise<void>
}

export function useSessionLoader(selectedAgent: string): SessionState {
  const [messages, setMessages] = useState<ChatMessage[]>([])
  const [sessionId, setSessionId] = useState<string | null>(null)
  const [sessionTitle, setSessionTitle] = useState('')
  const [sessionAgent, setSessionAgent] = useState<string | null>(null)
  const [loading, setLoading] = useState(true)
  const [reloadKey, setReloadKey] = useState(0)

  const load = useCallback(async () => {
    setLoading(true)
    const resp = await getCurrentSession()
    if (resp.success && resp.data && resp.data.id) {
      const sessionAgentId = resp.data.agent_id || 'default'
      if (sessionAgentId !== selectedAgent) {
        const sessionsResp = await listSessions()
        const agentSessions = (sessionsResp.data || [])
          .filter((s: any) => (s.agent_id || 'default') === selectedAgent)
          .sort((a: any, b: any) => b.created_at - a.created_at)
        if (agentSessions.length > 0) {
          const recent = agentSessions[0]
          await switchSession(recent.id)
          const sessionResp = await getCurrentSession()
          if (sessionResp.success && sessionResp.data) {
            setSessionId(sessionResp.data.id)
            setSessionTitle(sessionResp.data.title || 'Untitled')
            const agent = sessionResp.data.agent_id || null
            if (agent) setSessionAgent(agent)
            setMessages(deserializeMessages(sessionResp.data.messages || []))
          }
        } else {
          const createResp = await createSession(
            selectedAgent !== 'default' ? selectedAgent : undefined
          )
          if (createResp.success && createResp.data) {
            setSessionId(createResp.data.id)
            setSessionTitle('New Chat')
            setSessionAgent(createResp.data.agent_id || null)
            setMessages([])
          }
        }
        setLoading(false)
        return
      }
      setSessionId(resp.data.id)
      setSessionTitle(resp.data.title || 'Untitled')
      const agent = resp.data.agent_id || null
      if (agent) setSessionAgent(agent)
      setMessages(deserializeMessages(resp.data.messages || []))
    } else {
      const sessionsResp = await listSessions()
      const agentSessions = (sessionsResp.data || [])
        .filter((s: any) => (s.agent_id || 'default') === selectedAgent)
        .sort((a: any, b: any) => b.created_at - a.created_at)
      if (agentSessions.length > 0) {
        const recent = agentSessions[0]
        await switchSession(recent.id)
        const sessionResp = await getCurrentSession()
        if (sessionResp.success && sessionResp.data) {
          setSessionId(sessionResp.data.id)
          setSessionTitle(sessionResp.data.title || 'Untitled')
          const agent = sessionResp.data.agent_id || null
          if (agent) setSessionAgent(agent)
          setMessages(deserializeMessages(sessionResp.data.messages || []))
        }
      } else {
        const createResp = await createSession(
          selectedAgent !== 'default' ? selectedAgent : undefined
        )
        if (createResp.success && createResp.data) {
          setSessionId(createResp.data.id)
          setSessionTitle('New Chat')
          setSessionAgent(createResp.data.agent_id || null)
          setMessages([])
        }
      }
    }
    setLoading(false)
  }, [selectedAgent])

  useEffect(() => { load() }, [load, reloadKey])

  const newChat = useCallback(async () => {
    const createResp = await createSession(
      selectedAgent !== 'default' ? selectedAgent : undefined
    )
    if (createResp.success && createResp.data) {
      setSessionId(createResp.data.id)
      setSessionTitle('New Chat')
      setSessionAgent(createResp.data.agent_id || null)
      setMessages([])
    }
  }, [selectedAgent])

  return {
    messages,
    setMessages,
    sessionId,
    setSessionId,
    sessionTitle,
    sessionAgent,
    loading,
    reload: () => setReloadKey((k) => k + 1),
    newChat,
  }
}