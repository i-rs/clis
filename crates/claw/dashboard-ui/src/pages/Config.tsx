import { useState, useEffect } from 'react'
import { Settings, Bot, Eye, EyeOff, Check, AlertTriangle, Loader, Info } from 'lucide-react'
import { getAgentConfig, updateAgent, type AgentInfo } from '../api'

interface Props {
  selectedAgent: string
  onAgentsChange?: () => void
}

export default function ConfigPage({ selectedAgent, onAgentsChange }: Props) {
  const [config, setConfig] = useState<AgentInfo | null>(null)
  const [loading, setLoading] = useState(true)
  const [saving, setSaving] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [success, setSuccess] = useState(false)

  // Editable fields (only for non-default agents)
  const [provider, setProvider] = useState('')
  const [model, setModel] = useState('')
  const [baseUrl, setBaseUrl] = useState('')
  const [apiKey, setApiKey] = useState('')
  const [systemPrompt, setSystemPrompt] = useState('')

  const isDefault = selectedAgent === 'default'

  useEffect(() => {
    setLoading(true)
    setError(null)
    setSuccess(false)
    getAgentConfig(selectedAgent).then((resp) => {
      if (resp.success && resp.data) {
        const c = resp.data
        setConfig(c)
        setProvider(c.provider)
        setModel(c.model)
        setBaseUrl(c.base_url)
        setSystemPrompt(c.system_prompt || '')
      }
      setLoading(false)
    }).catch(() => setLoading(false))
  }, [selectedAgent])

  const handleSave = async () => {
    setSaving(true)
    setError(null)
    setSuccess(false)

    const body: Record<string, unknown> = {}
    if (provider !== config?.provider) body.provider = provider
    if (model !== config?.model) body.model = model
    if (baseUrl !== config?.base_url) body.base_url = baseUrl
    if (apiKey) body.api_key = apiKey
    if (systemPrompt !== (config?.system_prompt || '')) {
      body.system_prompt = systemPrompt || null
    }

    if (Object.keys(body).length === 0) {
      setSaving(false)
      return
    }

    const resp = await updateAgent(selectedAgent, body)
    if (resp.success) {
      setSuccess(true)
      setApiKey('')
      onAgentsChange?.()
      setTimeout(() => setSuccess(false), 3000)
    } else {
      setError(resp.error || 'Failed to save config')
    }
    setSaving(false)
  }

  return (
    <>
      <div className="page-header">
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <Bot size={16} style={{ color: 'var(--accent)' }} />
          <h2 style={{ fontSize: '15px' }}>
            {selectedAgent}
            {isDefault && <span className="badge info" style={{ marginLeft: '8px', fontSize: '10px' }}>Default</span>}
          </h2>
        </div>
      </div>

      <div className="page-body">
        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading config...
          </div>
        ) : config ? (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '16px', maxWidth: '640px' }}>
            {/* Agent info */}
            <div className="card" style={{ padding: '12px 16px' }}>
              <div style={{ fontSize: '12px', color: 'var(--text-muted)', lineHeight: '1.6' }}>
                {isDefault ? (
                  <span>The <strong>default</strong> agent uses the global configuration. To customize, create a new agent.</span>
                ) : (
                  <span>Configure agent-specific overrides. Fields left empty will inherit global defaults.</span>
                )}
              </div>
            </div>

            {!isDefault && (
              <>
                {/* Model Configuration */}
                <div className="card">
                  <div className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <Settings size={14} />
                    Model Configuration
                  </div>
                  <div className="card-body" style={{ marginTop: '12px' }}>
                    <ConfigField label="Provider" value={provider} onChange={setProvider} placeholder="e.g. openai" />
                    <ConfigField label="Model" value={model} onChange={setModel} placeholder="e.g. gpt-4o" />
                    <ConfigField label="Base URL" value={baseUrl} onChange={setBaseUrl} placeholder="e.g. https://api.openai.com/v1" />
                    <ConfigField label="API Key" value={apiKey} onChange={setApiKey} placeholder="Leave empty to keep existing" type="password" />
                  </div>
                </div>

                {/* System Prompt */}
                <div className="card">
                  <div className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                    <Info size={14} />
                    System Prompt
                  </div>
                  <div className="card-body" style={{ marginTop: '12px' }}>
                    <textarea
                      className="chat-input"
                      value={systemPrompt}
                      onChange={(e) => setSystemPrompt(e.target.value)}
                      placeholder="Custom system prompt (optional — leave empty for auto-generated default)"
                      rows={5}
                      style={{ width: '100%', minHeight: '100px', fontFamily: 'monospace', fontSize: '13px', lineHeight: '1.5' }}
                    />
                  </div>
                </div>

                {/* Enabled Tools */}
                {config.enabled_tools && config.enabled_tools.length > 0 && (
                  <div className="card">
                    <div className="card-title" style={{ display: 'flex', alignItems: 'center', gap: '6px' }}>
                      <Settings size={14} />
                      Enabled Tools ({config.enabled_tools.length})
                    </div>
                    <div className="card-body" style={{ marginTop: '8px', display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                      {config.enabled_tools.map((tool) => (
                        <span key={tool} className="badge enabled">{tool}</span>
                      ))}
                    </div>
                  </div>
                )}
              </>
            )}

            {/* For default agent: read-only display */}
            {isDefault && (
              <div className="card">
                <div className="card-title">Current Configuration</div>
                <table className="data-table" style={{ marginTop: '8px' }}>
                  <tbody>
                    <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Provider</td><td>{config.provider}</td></tr>
                    <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Model</td><td>{config.model}</td></tr>
                    <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Base URL</td><td style={{ fontFamily: 'monospace', fontSize: '12px' }}>{config.base_url}</td></tr>
                    <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Tools</td>
                      <td>
                        {config.enabled_tools && config.enabled_tools.length > 0 ? (
                          <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                            {config.enabled_tools.map((t) => <span key={t} className="badge enabled">{t}</span>)}
                          </div>
                        ) : (
                          <span style={{ color: 'var(--text-muted)', fontStyle: 'italic' }}>All tools enabled</span>
                        )}
                      </td>
                    </tr>
                  </tbody>
                </table>
              </div>
            )}

            {/* Status messages */}
            {error && (
              <div style={{
                padding: '10px 14px',
                borderRadius: 'var(--radius)',
                background: 'var(--error-bg)',
                color: 'var(--error)',
                border: '1px solid rgba(255,107,107,0.3)',
                fontSize: '13px',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
              }}>
                <AlertTriangle size={14} />
                {error}
              </div>
            )}
            {success && (
              <div style={{
                padding: '10px 14px',
                borderRadius: 'var(--radius)',
                background: 'var(--success-bg)',
                color: 'var(--success)',
                border: '1px solid rgba(81,207,102,0.3)',
                fontSize: '13px',
                display: 'flex',
                alignItems: 'center',
                gap: '8px',
              }}>
                <Check size={14} />
                Configuration saved
              </div>
            )}

            {/* Save button */}
            {!isDefault && (
              <div style={{ display: 'flex', justifyContent: 'flex-end', paddingTop: '4px' }}>
                <button
                  className="send-btn btn-sm"
                  onClick={handleSave}
                  disabled={saving}
                >
                  {saving ? (
                    <><Loader size={14} className="loading-spinner" /> Saving...</>
                  ) : (
                    <><Check size={14} /> Save Changes</>
                  )}
                </button>
              </div>
            )}
          </div>
        ) : (
          <div className="empty-state">
            <Settings size={40} className="empty-state-icon" />
            <p>Failed to load configuration for agent <strong>{selectedAgent}</strong>.</p>
          </div>
        )}
      </div>
    </>
  )
}

// ── Config Field ──

function ConfigField({
  label,
  value,
  onChange,
  placeholder,
  type,
}: {
  label: string
  value: string
  onChange: (v: string) => void
  placeholder?: string
  type?: string
}) {
  const [showPassword, setShowPassword] = useState(false)
  const isPassword = type === 'password'
  const inputType = isPassword && !showPassword ? 'password' : 'text'

  return (
    <div style={{ marginBottom: '10px' }}>
      <label style={{
        display: 'block',
        fontSize: '11px',
        fontWeight: 600,
        color: 'var(--text-muted)',
        marginBottom: '3px',
        textTransform: 'uppercase',
        letterSpacing: '0.04em',
      }}>
        {label}
        <span style={{ fontWeight: 400, textTransform: 'none', color: 'var(--text-muted)', marginLeft: '6px', fontSize: '10px' }}>
          (optional)
        </span>
      </label>
      <div style={{ position: 'relative' }}>
        <input
          type={inputType}
          className="chat-input"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          style={{ width: '100%', paddingRight: isPassword ? '36px' : undefined }}
        />
        {isPassword && (
          <button
            className="btn-ghost"
            onClick={() => setShowPassword(!showPassword)}
            style={{ position: 'absolute', right: '4px', top: '50%', transform: 'translateY(-50%)', padding: '4px' }}
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
