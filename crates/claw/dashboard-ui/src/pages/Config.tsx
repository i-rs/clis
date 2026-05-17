import { useState, useEffect } from 'react'
import { Settings } from 'lucide-react'
import { getConfig } from '../api'

export default function ConfigPage() {
  const [config, setConfig] = useState<Record<string, unknown> | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    getConfig().then((resp) => {
      if (resp.success && resp.data) {
        setConfig(resp.data)
      }
      setLoading(false)
    }).catch(() => setLoading(false))
  }, [])

  return (
    <>
      <div className="page-header">
        <h2>Configuration</h2>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading config...
          </div>
        ) : config ? (
          <div className="card">
            <table className="data-table">
              <thead>
                <tr>
                  <th>Key</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                {Object.entries(config).map(([key, value]) => (
                  <tr key={key}>
                    <td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{key}</td>
                    <td>
                      {Array.isArray(value) ? (
                        <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap' }}>
                          {(value as string[]).map((v, i) => (
                            <span key={i} className="badge enabled">{v}</span>
                          ))}
                        </div>
                      ) : typeof value === 'object' && value !== null ? (
                        <pre>{JSON.stringify(value, null, 2)}</pre>
                      ) : (
                        String(value)
                      )}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="empty-state">
            <Settings size={40} className="empty-state-icon" />
            <p>Failed to load configuration.</p>
          </div>
        )}
      </div>
    </>
  )
}
