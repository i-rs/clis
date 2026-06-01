import { useState, useEffect } from 'react'
import { Puzzle } from 'lucide-react'
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
        <div className="page-header-left">
          <div className="page-header-icon">
            <Puzzle size={16} />
          </div>
          <h2>Plugins ({plugins.length})</h2>
        </div>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading plugins...
          </div>
        ) : plugins.length === 0 ? (
          <div className="empty-state">
            <div className="empty-state-icon">
              <Puzzle size={24} />
            </div>
            <h3>No plugins found</h3>
            <p>Add plugin manifests to <code>~/.i-rs/claw/plugins/</code></p>
          </div>
        ) : (
          <div className="table-wrapper">
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
                    <td>v{plugin.version}</td>
                    <td>{plugin.description}</td>
                    <td>{plugin.author || '-'}</td>
                    <td>
                      <span className={`badge ${plugin.enabled ? 'badge-enabled' : 'badge-disabled'}`}>
                        {plugin.enabled ? 'Enabled' : 'Disabled'}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </>
  )
}
