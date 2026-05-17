import { useState, useEffect } from 'react'
import { listPlugins, type PluginInfo } from '../api'

export default function PluginsPage() {
  const [plugins, setPlugins] = useState<PluginInfo[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    listPlugins().then((resp) => {
      if (resp.success && resp.data) {
        setPlugins(resp.data)
      }
      setLoading(false)
    }).catch(() => setLoading(false))
  }, [])

  return (
    <>
      <div className="page-header">
        <h2>Plugins ({plugins.length})</h2>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">Loading plugins...</div>
        ) : plugins.length === 0 ? (
          <div className="empty-state">
            <p>No plugins discovered. Add plugin manifests to ~/.i-rs-claw/plugins/</p>
          </div>
        ) : (
          <table className="data-table">
            <thead>
              <tr>
                <th>Name</th>
                <th>Version</th>
                <th>Description</th>
                <th>Author</th>
                <th>Status</th>
              </tr>
            </thead>
            <tbody>
              {plugins.map((plugin) => (
                <tr key={plugin.name}>
                  <td style={{ fontWeight: 600, color: 'var(--text-primary)' }}>{plugin.name}</td>
                  <td>{plugin.version}</td>
                  <td>{plugin.description}</td>
                  <td>{plugin.author || '-'}</td>
                  <td>
                    <span className={`badge ${plugin.enabled ? 'enabled' : 'disabled'}`}>
                      {plugin.enabled ? 'Enabled' : 'Disabled'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
    </>
  )
}
