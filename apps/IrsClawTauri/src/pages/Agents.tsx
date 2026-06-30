import { useEffect, useState } from 'react'
import { Bot, Trash2, Plus } from 'lucide-react'
import { listAgents, deleteAgent, createAgent } from '../api/agents'
import type { AgentInfo } from '../api/types'
import EmptyState from '../components/common/EmptyState'
import { toast } from '../components/common/Toast'

export default function AgentsPage() {
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [showForm, setShowForm] = useState(false)
  const [newId, setNewId] = useState('')
  const [newModel, setNewModel] = useState('')

  const refresh = async () => {
    setLoading(true)
    const r = await listAgents()
    if (r.success && r.data) setAgents(r.data)
    setLoading(false)
  }
  useEffect(() => { refresh() }, [])

  const handleDelete = async (id: string) => {
    await deleteAgent(id)
    toast('Agent deleted', 'success')
    refresh()
  }

  const handleCreate = async () => {
    if (!newId.trim()) return
    const r = await createAgent({ id: newId.trim(), model: newModel.trim() || undefined })
    if (r.success) { toast('Agent created', 'success'); setNewId(''); setNewModel(''); setShowForm(false); refresh() }
    else toast(r.error || 'Failed', 'error')
  }

  if (loading) return <div className="loading"><div className="loading-spinner" /></div>
  if (agents.length === 0) return <EmptyState icon={<Bot size={28} />} title="No Agents" action={<button className="btn btn-primary" onClick={() => setShowForm(true)}><Plus size={14} />New Agent</button>} />

  return (
    <div style={{ padding: 24, overflowY: 'auto', flex: 1 }}>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 20 }}>
        <h2 style={{ fontSize: 18, fontWeight: 700 }}>Agents</h2>
        <button className="btn btn-primary btn-sm" onClick={() => setShowForm(!showForm)}><Plus size={14} />New</button>
      </div>
      {showForm && (
        <div className="card" style={{ marginBottom: 16 }}>
          <div style={{ display: 'flex', gap: 8, flexDirection: 'column' }}>
            <input className="config-input" value={newId} onChange={(e) => setNewId(e.target.value)} placeholder="Agent ID" />
            <input className="config-input" value={newModel} onChange={(e) => setNewModel(e.target.value)} placeholder="Model (e.g. gpt-4o)" />
            <button className="btn btn-primary btn-sm" onClick={handleCreate}>Create</button>
          </div>
        </div>
      )}
      {agents.map((a) => (
        <div key={a.id} className="card" style={{ marginBottom: 10, display: 'flex', alignItems: 'center', gap: 14 }}>
          <div className="session-icon"><Bot size={18} /></div>
          <div style={{ flex: 1 }}>
            <div style={{ fontWeight: 600, fontSize: 14 }}>{a.id} {a.is_sub_agent && <span className="badge badge-info">sub</span>}</div>
            <div style={{ fontSize: 12, color: 'var(--text-muted)' }}>{a.provider} · {a.model} · {a.tool_count} tools</div>
          </div>
          {!a.is_sub_agent && a.id !== 'default' && <button className="session-btn session-btn-delete" onClick={() => handleDelete(a.id)}><Trash2 size={12} /></button>}
        </div>
      ))}
    </div>
  )
}
