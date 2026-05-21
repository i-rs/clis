import { useState, useEffect } from 'react'
import { Wrench, Terminal, Search, Folder, Cloud, Code, Database } from 'lucide-react'
import { listTools, type ToolSchema } from '../api'

const CATEGORY_ICONS: Record<string, { icon: React.ReactNode; color: string }> = {
  'i-rs': { icon: <Terminal size={16} />, color: 'var(--accent)' },
  'filesystem': { icon: <Folder size={16} />, color: 'var(--info)' },
  'web': { icon: <Cloud size={16} />, color: 'var(--success)' },
  'developer': { icon: <Code size={16} />, color: 'var(--warning)' },
  'data': { icon: <Database size={16} />, color: 'var(--ai-primary)' },
  'default': { icon: <Wrench size={16} />, color: 'var(--text-muted)' },
}

function getToolCategory(name: string): string {
  const lower = name.toLowerCase()
  if (lower.includes('i_rs') || lower.includes('irs')) return 'i-rs'
  if (lower.includes('file') || lower.includes('path')) return 'filesystem'
  if (lower.includes('web') || lower.includes('http') || lower.includes('search')) return 'web'
  if (lower.includes('code') || lower.includes('git') || lower.includes('exec')) return 'developer'
  if (lower.includes('db') || lower.includes('sql') || lower.includes('store')) return 'data'
  return 'default'
}

export default function ToolsPage() {
  const [tools, setTools] = useState<ToolSchema[]>([])
  const [loading, setLoading] = useState(true)
  const [filter, setFilter] = useState('')

  useEffect(() => {
    listTools().then((resp) => {
      if (resp.success && resp.data) {
        setTools(resp.data)
      }
      setLoading(false)
    }).catch(() => setLoading(false))
  }, [])

  const filteredTools = tools.filter(tool => 
    tool.function.name.toLowerCase().includes(filter.toLowerCase()) ||
    tool.function.description.toLowerCase().includes(filter.toLowerCase())
  )

  return (
    <>
      <div className="page-header">
        <div className="page-header-left">
          <div className="page-header-icon">
            <Wrench size={16} />
          </div>
          <h2>Tools ({filteredTools.length})</h2>
        </div>
        <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
          <div style={{ position: 'relative' }}>
            <Search size={14} style={{ position: 'absolute', left: '12px', top: '50%', transform: 'translateY(-50%)', color: 'var(--text-muted)' }} />
            <input
              type="text"
              className="config-input"
              placeholder="Search tools..."
              value={filter}
              onChange={(e) => setFilter(e.target.value)}
              style={{ paddingLeft: '36px', width: '200px' }}
            />
          </div>
        </div>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading tools...
          </div>
        ) : filteredTools.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <Wrench size={24} />
            </div>
            <h3>No tools found</h3>
            <p>Try adjusting your search.</p>
          </div>
        ) : (
          <div className="tools-grid">
            {filteredTools.map((tool) => {
              const category = getToolCategory(tool.function.name)
              const categoryInfo = CATEGORY_ICONS[category] || CATEGORY_ICONS.default
              return (
                <div key={tool.function.name} className="tool-card">
                  <div className="tool-card-header">
                    <div className="tool-card-icon" style={{ background: `${categoryInfo.color}20`, color: categoryInfo.color }}>
                      {categoryInfo.icon}
                    </div>
                    <div>
                      <div className="tool-card-title">{tool.function.name}</div>
                      <div className="tool-card-category">{category}</div>
                    </div>
                  </div>
                  <div className="tool-card-description">{tool.function.description}</div>
                  {tool.function.parameters && Object.keys(tool.function.parameters).length > 0 && (
                    <details style={{ marginTop: '12px' }}>
                      <summary style={{ cursor: 'pointer', fontSize: '11px', color: 'var(--text-muted)', fontWeight: 600 }}>
                        Parameters ({Object.keys(tool.function.parameters).length})
                      </summary>
                      <pre style={{ marginTop: '8px', fontSize: '11px' }}>
                        {JSON.stringify(tool.function.parameters, null, 2)}
                      </pre>
                    </details>
                  )}
                </div>
              )
            })}
          </div>
        )}
      </div>
    </>
  )
}
