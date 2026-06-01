import { useState, useEffect } from 'react'
import { Bot, Plus, Trash2, X, Check, AlertTriangle, Eye, EyeOff } from 'lucide-react'
import { listAgents, createAgent, deleteAgent, type AgentInfo } from '../api'

interface Props {
  onAgentsChange?: () => void
}

export default function AgentsPage({ onAgentsChange }: Props) {
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [showCreate, setShowCreate] = useState(false)
  const [submitting, setSubmitting] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const [formId, setFormId] = useState('')
  const [formProvider, setFormProvider] = useState('')
  const [formModel, setFormModel] = useState('')
  const [formBaseUrl, setFormBaseUrl] = useState('')
  const [formApiKey, setFormApiKey] = useState('')
  const [formSystemPrompt, setFormSystemPrompt] = useState('')
  const [formTools, setFormTools] = useState('')

  const loadAgents = () => {
    setLoading(true)
    listAgents().then((resp) => {
      if (resp.success && resp.data) {
        setAgents(resp.data.filter((a) => !a.is_sub_agent))
      }
      setLoading(false)
    }).catch(() => setLoading(false))
  }

  useEffect(() => {
    loadAgents()
  }, [])

  const resetForm = () => {
    setFormId('')
    setFormProvider('')
    setFormModel('')
    setFormBaseUrl('')
    setFormApiKey('')
    setFormSystemPrompt('')
    setFormTools('')
    setError(null)
  }

  const handleCreate = async () => {
    const id = formId.trim()
    if (!id) {
      setError('Agent ID is required')
      return
    }
    if (id === 'default') {
      setError('Cannot create agent with id "default"')
      return
    }

    setSubmitting(true)
    setError(null)

    const body: Record<string, unknown> = { id }
    if (formProvider.trim()) body.provider = formProvider.trim()
    if (formModel.trim()) body.model = formModel.trim()
    if (formBaseUrl.trim()) body.base_url = formBaseUrl.trim()
    if (formApiKey.trim()) body.api_key = formApiKey.trim()
    if (formSystemPrompt.trim()) body.system_prompt = formSystemPrompt.trim()
    if (formTools.trim()) {
      body.enabled_tools = formTools.split(',').map((t) => t.trim()).filter(Boolean)
    }

    const resp = await createAgent(body)
    if (resp.success) {
      setShowCreate(false)
      resetForm()
      loadAgents()
      onAgentsChange?.()
    } else {
      setError(resp.error || 'Failed to create agent')
    }
    setSubmitting(false)
  }

  const handleDelete = async (id: string) => {
    if (!window.confirm(`Are you sure you want to delete agent "${id}"?\nThis action cannot be undone.`)) return

    const resp = await deleteAgent(id)
    if (resp.success) {
      loadAgents()
      onAgentsChange?.()
    } else {
      setError(resp.error || 'Failed to delete agent')
    }
  }

  return (
    <>
      <div className="page-header">
        <div className="page-header-left">
          <div className="page-header-icon">
            <Bot size={16} />
          </div>
          <h2>Agents ({agents.length})</h2>
        </div>
        <button className="btn btn-primary btn-sm" onClick={() => { resetForm(); setShowCreate(true); }}>
          <Plus size={14} />
          Create Agent
        </button>
      </div>

      <div className="page-body">
        {error && (
          <div className="status-message error" style={{ marginBottom: '16px' }}>
            <AlertTriangle size={16} />
            {error}
          </div>
        )}

        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading agents...
          </div>
        ) : agents.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <Bot size={24} />
            </div>
            <h3>No agents configured</h3>
            <p>Create an agent to get started.</p>
          </div>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {agents.map((agent) => (
              <AgentCard
                key={agent.id}
                agent={agent}
                onDelete={agent.id !== 'default' ? handleDelete : undefined}
              />
            ))}
          </div>
        )}

        {showCreate && (
          <div style={{
            position: 'fixed',
            inset: 0,
            background: 'rgba(0,0,0,0.6)',
            backdropFilter: 'blur(4px)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }} onClick={() => { if (!submitting) setShowCreate(false) }}>
            <div style={{
              background: 'var(--bg-elevated)',
              border: '1px solid var(--border)',
              borderRadius: 'var(--radius-xl)',
              padding: '24px',
              width: '520px',
              maxHeight: '80vh',
              overflowY: 'auto',
            }} onClick={(e) => e.stopPropagation()}>
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
                <h3 style={{ fontSize: '16px', fontWeight: 600, color: 'var(--text-primary)' }}>Create New Agent</h3>
                <button className="btn btn-ghost btn-icon" onClick={() => { if (!submitting) setShowCreate(false) }} disabled={submitting}>
                  <X size={16} />
                </button>
              </div>

              <FormField label="Agent ID *" value={formId} onChange={setFormId} placeholder="e.g. my-agent" disabled={submitting} />
              <FormField label="Provider" value={formProvider} onChange={setFormProvider} placeholder="e.g. openai" disabled={submitting} />
              <FormField label="Model" value={formModel} onChange={setFormModel} placeholder="e.g. gpt-4o" disabled={submitting} />
              <FormField label="Base URL" value={formBaseUrl} onChange={setFormBaseUrl} placeholder="e.g. https://api.openai.com/v1" disabled={submitting} />
              <FormField label="API Key" value={formApiKey} onChange={setFormApiKey} placeholder="sk-..." type="password" disabled={submitting} />
              <div style={{ marginBottom: '12px' }}>
                <label style={{ display: 'block', fontSize: '11px', fontWeight: 700, color: 'var(--text-muted)', marginBottom: '6px', textTransform: 'uppercase', letterSpacing: '0.06em' }}>
                  System Prompt
                </label>
                <textarea
                  className="config-input config-textarea"
                  value={formSystemPrompt}
                  onChange={(e) => setFormSystemPrompt(e.target.value)}
                  placeholder="Custom system prompt (optional)"
                  disabled={submitting}
                  rows={4}
                />
              </div>
              <FormField label="Enabled Tools (comma-separated)" value={formTools} onChange={setFormTools} placeholder="e.g. i_rs, web_search (empty=all)" disabled={submitting} />

              <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end', marginTop: '20px', paddingTop: '16px', borderTop: '1px solid var(--border)' }}>
                <button
                  className="btn btn-secondary"
                  onClick={() => { if (!submitting) setShowCreate(false) }}
                  disabled={submitting}
                >
                  Cancel
                </button>
                <button
                  className="btn btn-primary"
                  onClick={handleCreate}
                  disabled={submitting || !formId.trim()}
                >
                  {submitting ? (
                    <><Loader size={14} /> Creating...</>
                  ) : (
                    <><Check size={14} /> Create</>
                  )}
                </button>
              </div>
            </div>
          </div>
        )}
      </div>
    </>
  )
}

function AgentCard({ agent, onDelete }: { agent: AgentInfo; onDelete?: (id: string) => void }) {
  const [expanded, setExpanded] = useState(false)

  return (
    <div className="card" style={{ padding: 0, overflow: 'hidden' }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '14px 16px',
          cursor: 'pointer',
          userSelect: 'none',
        }}
        onClick={() => setExpanded(!expanded)}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
          <div style={{
            width: '36px',
            height: '36px',
            borderRadius: '10px',
            background: 'var(--ai-muted)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: 'var(--ai-primary)',
          }}>
            <Bot size={18} />
          </div>
          <div>
            <div style={{ fontSize: '14px', fontWeight: 600, color: 'var(--text-primary)' }}>
              {agent.id}
              {agent.id === 'default' && (
                <span className="badge badge-info" style={{ marginLeft: '8px', fontSize: '10px' }}>Default</span>
              )}
            </div>
            <div style={{ fontSize: '12px', color: 'var(--text-muted)', marginTop: '2px' }}>
              {agent.provider} · {agent.model} · {agent.tool_count} tool{agent.tool_count !== 1 ? 's' : ''}
            </div>
          </div>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          {onDelete && (
            <button
              className="btn btn-ghost btn-icon"
              onClick={(e) => { e.stopPropagation(); onDelete(agent.id) }}
              title="Delete agent"
              style={{ color: 'var(--error)' }}
            >
              <Trash2 size={14} />
            </button>
          )}
        </div>
      </div>

      {expanded && (
        <div style={{ borderTop: '1px solid var(--border)', padding: '14px 16px' }}>
          {agent.system_prompt ? (
            <div style={{ marginBottom: '10px' }}>
              <div style={{ fontSize: '11px', fontWeight: 700, color: 'var(--text-muted)', textTransform: 'uppercase', letterSpacing: '0.06em', marginBottom: '6px' }}>
                System Prompt
              </div>
              <pre style={{
                background: 'var(--bg-tertiary)',
                border: '1px solid var(--border)',
                borderRadius: 'var(--radius)',
                padding: '12px',
                fontSize: '12px',
                lineHeight: 1.5,
                overflowX: 'auto',
                whiteSpace: 'pre-wrap',
                wordBreak: 'break-word',
                color: 'var(--text-secondary)',
                margin: 0,
                maxHeight: '200px',
                overflowY: 'auto',
              }}>{agent.system_prompt}</pre>
            </div>
          ) : (
            <div style={{ fontSize: '12px', color: 'var(--text-muted)', fontStyle: 'italic' }}>
              No custom system prompt (uses auto-generated default)
            </div>
          )}
        </div>
      )}
    </div>
  )
}

function FormField({
  label,
  value,
  onChange,
  placeholder,
  type,
  disabled,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  type?: string
  disabled?: boolean
}) {
  const [showPassword, setShowPassword] = useState(false)
  const isPassword = type === 'password'
  const inputType = isPassword && !showPassword ? 'password' : 'text'

  return (
    <div style={{ marginBottom: '12px' }}>
      <label style={{
        display: 'block',
        fontSize: '11px',
        fontWeight: 700,
        color: 'var(--text-muted)',
        marginBottom: '6px',
        textTransform: 'uppercase',
        letterSpacing: '0.06em',
      }}>
        {label}
      </label>
      <div style={{ position: 'relative' }}>
        <input
          type={inputType}
          className="config-input"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          disabled={disabled}
          style={{ width: '100%', paddingRight: isPassword ? '40px' : undefined }}
        />
        {isPassword && (
          <button
            className="btn btn-ghost btn-icon"
            onClick={() => setShowPassword(!showPassword)}
            style={{ position: 'absolute', right: '4px', top: '50%', transform: 'translateY(-50%)' }}
            type="button"
            tabIndex={-1}
          >
            {showPassword ? <EyeOff size={14} /> : <Eye size={14} />}
          </button>
        )}
      </div>
    </div>
  )
}

function Loader({ size = 14 }: { size?: number }) {
  return (
    <div style={{
      width: size,
      height: size,
      border: '2px solid var(--border)',
      borderTopColor: 'var(--accent)',
      borderRadius: '50%',
      animation: 'spin 0.6s linear infinite',
    }} />
  )
}
