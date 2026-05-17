import { useState, useEffect } from 'react'
import { Wrench } from 'lucide-react'
import { listTools, type ToolSchema } from '../api'

export default function ToolsPage() {
  const [tools, setTools] = useState<ToolSchema[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    listTools().then((resp) => {
      if (resp.success && resp.data) {
        setTools(resp.data)
      }
      setLoading(false)
    }).catch(() => setLoading(false))
  }, [])

  return (
    <>
      <div className="page-header">
        <h2>Tools ({tools.length})</h2>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading tools...
          </div>
        ) : tools.length === 0 ? (
          <div className="empty-state">
            <Wrench size={40} className="empty-state-icon" />
            <p>No tools available.</p>
          </div>
        ) : (
          <div>
            {tools.map((tool) => (
              <div key={tool.function.name} className="card">
                <div className="card-title">{tool.function.name}</div>
                <div className="card-body">
                  <p style={{ marginBottom: '8px' }}>{tool.function.description}</p>
                  {tool.function.parameters && Object.keys(tool.function.parameters).length > 0 && (
                    <details>
                      <summary style={{ cursor: 'pointer', fontSize: '12px', color: 'var(--text-muted)' }}>
                        Parameters
                      </summary>
                      <pre style={{ marginTop: '8px' }}>
                        {JSON.stringify(tool.function.parameters, null, 2)}
                      </pre>
                    </details>
                  )}
                </div>
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  )
}
