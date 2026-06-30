import { useEffect, useState } from 'react'
import { BarChart3, Zap, DollarSign, Activity } from 'lucide-react'
import { getStats } from '../api/stats'
import type { StatsResponse } from '../api/types'
import EmptyState from '../components/common/EmptyState'

const PERIODS = ['today', 'week', 'month', 'all']

export default function UsagePage() {
  const [period, setPeriod] = useState('all')
  const [stats, setStats] = useState<StatsResponse | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    setLoading(true)
    getStats(period).then((r) => { if (r.success && r.data) setStats(r.data); setLoading(false) })
  }, [period])

  if (loading) return <div className="loading"><div className="loading-spinner" /></div>
  if (!stats) return <EmptyState icon={<BarChart3 size={28} />} title="No Usage Data" />

  return (
    <div style={{ padding: 24, overflowY: 'auto', flex: 1 }}>
      <h2 style={{ fontSize: 18, fontWeight: 700, marginBottom: 20 }}>Token Usage</h2>
      <div className="period-selector">
        {PERIODS.map((p) => <button key={p} className={`period-btn ${p === period ? 'active' : ''}`} onClick={() => setPeriod(p)}>{p}</button>)}
      </div>
      <div className="usage-cards">
        <div className="usage-card">
          <div className="usage-card-icon"><Activity size={22} /></div>
          <div className="usage-card-content"><div className="usage-card-value">{stats.total_requests}</div><div className="usage-card-label">Total Requests</div></div>
        </div>
        <div className="usage-card">
          <div className="usage-card-icon"><Zap size={22} /></div>
          <div className="usage-card-content"><div className="usage-card-value">{stats.total_tokens.toLocaleString()}</div><div className="usage-card-label">Total Tokens</div></div>
        </div>
        <div className="usage-card">
          <div className="usage-card-icon"><DollarSign size={22} /></div>
          <div className="usage-card-content"><div className="usage-card-value">${stats.total_cost_usd.toFixed(4)}</div><div className="usage-card-label">Total Cost</div></div>
        </div>
      </div>
      <div className="card" style={{ marginTop: 16 }}>
        <div className="card-title">Today</div>
        <div className="today-stats" style={{ marginTop: 12 }}>
          <div className="today-stat"><div className="today-stat-value">{stats.today.requests}</div><div className="today-stat-label">Requests</div></div>
          <div className="today-stat"><div className="today-stat-value">{stats.today.tokens.toLocaleString()}</div><div className="today-stat-label">Tokens</div></div>
          <div className="today-stat"><div className="today-stat-value">${stats.today.cost_usd.toFixed(4)}</div><div className="today-stat-label">Cost</div></div>
        </div>
      </div>
    </div>
  )
}
