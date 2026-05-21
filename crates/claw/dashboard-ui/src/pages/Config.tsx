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
        <div className="page-header-left">
          <div className="page-header-icon">
            <Bot size={16} />
          </div>
          <h2>
            Agent: {selectedAgent}
            {isDefault && <span className="badge badge-info" style={{ marginLeft: '8px', fontSize: '10px' }}>Default</span>}
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
          <div className="config-container">
            <div className="config-section">
              <div className="config-section-header">
                <div className="config-section-icon">
                  <Info size={16} />
                </div>
                <div className="config-section-title">About Agent Configuration</div>
              </div>
              <div className="config-section-body">
                <p style={{ fontSize: '13px', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
                  {isDefault ? (
                    <>The <strong>default</strong> agent uses the global configuration from config.toml. Create a new agent to customize settings.</>
                  ) : (
                    <>Configure agent-specific overrides. Fields left empty will inherit global defaults from config.toml.</>
                  )}
                </p>
              </div>
            </div>

            {!isDefault && (
              <>
                <div className="config-section">
                  <div className="config-section-header">
                    <div className="config-section-icon">
                      <Settings size={16} />
                    </div>
                    <div className="config-section-title">Model Configuration</div>
                  </div>
                  <div className="config-section-body">
                    <ConfigField label="Provider" value={provider} onChange={setProvider} placeholder="e.g. openai" />
                    <ConfigField label="Model" value={model} onChange={setModel} placeholder="e.g. gpt-4o" />
                    <ConfigField label="Base URL" value={baseUrl} onChange={setBaseUrl} placeholder="e.g. https://api.openai.com/v1" />
                    <ConfigField label="API Key" value={apiKey} onChange={setApiKey} placeholder="Leave empty to keep existing" type="password" />
                  </div>
                </div>

                <div className="config-section">
                  <div className="config-section-header">
                    <div className="config-section-icon">
                      <Info size={16} />
                    </div>
                    <div className="config-section-title">System Prompt</div>
                  </div>
                  <div className="config-section-body">
                    <textarea
                      className="config-input config-textarea"
                      value={systemPrompt}
                      onChange={(e) => setSystemPrompt(e.target.value)}
                      placeholder="Custom system prompt (optional — leave empty for auto-generated default)"
                      rows={5}
                    />
                    <div className="config-hint">Leave empty to use the default auto-generated system prompt.</div>
                  </div>
                </div>

                {config.enabled_tools && config.enabled_tools.length > 0 && (
                  <div className="config-section">
                    <div className="config-section-header">
                      <div className="config-section-icon">
                        <Settings size={16} />
                      </div>
                      <div className="config-section-title">Enabled Tools ({config.enabled_tools.length})</div>
                    </div>
                    <div className="config-section-body">
                      <div style={{ display: 'flex', gap: '6px', flexWrap: 'wrap' }}>
                        {config.enabled_tools.map((tool) => (
                          <span key={tool} className="badge badge-enabled">{tool}</span>
                        ))}
                      </div>
                    </div>
                  </div>
                )}
              </>
            )}

            {isDefault && (
              <div className="config-section">
                <div className="config-section-header">
                  <div className="config-section-icon">
                    <Settings size={16} />
                  </div>
                  <div className="config-section-title">Current Configuration</div>
                </div>
                <div className="config-section-body">
                  <table className="data-table" style={{ marginTop: '8px' }}>
                    <tbody>
                      <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)', width: '120px' }}>Provider</td><td>{config.provider}</td></tr>
                      <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Model</td><td>{config.model}</td></tr>
                      <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Base URL</td><td style={{ fontFamily: 'monospace', fontSize: '12px' }}>{config.base_url}</td></tr>
                      <tr><td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>Tools</td>
                        <td>
                          {config.enabled_tools && config.enabled_tools.length > 0 ? (
                            <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                              {config.enabled_tools.map((t) => <span key={t} className="badge badge-enabled">{t}</span>)}
                            </div>
                          ) : (
                            <span style={{ color: 'var(--text-muted)', fontStyle: 'italic' }}>All tools enabled</span>
                          )}
                        </td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            )}

            {error && (
              <div className="status-message error">
                <AlertTriangle size={16} />
                {error}
              </div>
            )}
            {success && (
              <div className="status-message success">
                <Check size={16} />
                Configuration saved successfully
              </div>
            )}

            {!isDefault && (
              <div style={{ display: 'flex', justifyContent: 'flex-end', paddingTop: '8px' }}>
                <button
                  className="btn btn-primary"
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
            <div className="empty-state-icon">
              <Settings size={24} />
            </div>
            <h3>Failed to load</h3>
            <p>Could not load configuration for agent <strong>{selectedAgent}</strong>.</p>
          </div>
        )}
      </div>
    </>
  )
}

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
    <div className="config-field">
      <label className="config-label">
        {label}
        <span style={{ fontWeight: 400, textTransform: 'none', color: 'var(--text-muted)', marginLeft: '6px', fontSize: '10px' }}>
          (optional)
        </span>
      </label>
      <div style={{ position: 'relative' }}>
        <input
          type={inputType}
          className="config-input"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          style={{ paddingRight: isPassword ? '40px' : undefined }}
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
