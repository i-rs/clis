import { useState, useEffect } from 'react'
import { BarChart3, TrendingUp, DollarSign, Activity, RefreshCw } from 'lucide-react'
import { getStats, type StatsResponse } from '../api'

export default function UsagePage() {
  const [stats, setStats] = useState<StatsResponse | null>(null)
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [period, setPeriod] = useState<string>('all')

  const loadStats = async () => {
    setLoading(true)
    setError(null)
    try {
      const resp = await getStats(period)
      if (resp.success && resp.data) {
        setStats(resp.data)
      } else {
        setError(resp.error || 'Failed to load stats')
      }
    } catch (err) {
      setError('Network error')
    }
    setLoading(false)
  }

  useEffect(() => {
    loadStats()
  }, [period])

  const formatNumber = (n: number | undefined) => {
    if (n === undefined || n === null) return '0'
    return n.toLocaleString()
  }

  const formatCost = (c: number | undefined) => {
    if (c === undefined || c === null) return '$0.00'
    return '$' + c.toFixed(4)
  }

  return (
    <>
      <div className="page-header">
        <div className="page-header-left">
          <div className="page-header-icon">
            <BarChart3 size={16} />
          </div>
          <h2>Token Usage</h2>
        </div>
        <button className="btn btn-ghost btn-sm" onClick={loadStats} disabled={loading}>
          <RefreshCw size={14} className={loading ? 'animate-spin' : ''} />
          Refresh
        </button>
      </div>

      <div className="page-body">
        {/* Period Selector */}
        <div className="period-selector">
          <button
            className={`period-btn ${period === 'today' ? 'active' : ''}`}
            onClick={() => setPeriod('today')}
          >Today</button>
          <button
            className={`period-btn ${period === '7d' ? 'active' : ''}`}
            onClick={() => setPeriod('7d')}
          >7 Days</button>
          <button
            className={`period-btn ${period === '30d' ? 'active' : ''}`}
            onClick={() => setPeriod('30d')}
          >30 Days</button>
          <button
            className={`period-btn ${period === 'all' ? 'active' : ''}`}
            onClick={() => setPeriod('all')}
          >All Time</button>
        </div>

        {loading && !stats && (
          <div className="loading">
            <div className="loading-spinner" />
            Loading usage data...
          </div>
        )}

        {error && !stats && (
          <div className="empty-state">
            <p style={{ color: 'var(--red)' }}>{error}</p>
            <button className="btn btn-primary" onClick={loadStats}>Retry</button>
          </div>
        )}

        {stats && (
          <>
            {/* Summary Cards */}
            <div className="usage-cards">
              <div className="usage-card usage-card-primary">
                <div className="usage-card-icon">
                  <BarChart3 size={20} />
                </div>
                <div className="usage-card-content">
                  <div className="usage-card-value">{formatNumber(stats.total_tokens)}</div>
                  <div className="usage-card-label">Total Tokens</div>
                </div>
              </div>

              <div className="usage-card">
                <div className="usage-card-icon">
                  <Activity size={20} />
                </div>
                <div className="usage-card-content">
                  <div className="usage-card-value">{formatNumber(stats.total_requests)}</div>
                  <div className="usage-card-label">Total Requests</div>
                </div>
              </div>

              <div className="usage-card">
                <div className="usage-card-icon">
                  <DollarSign size={20} />
                </div>
                <div className="usage-card-content">
                  <div className="usage-card-value">{formatCost(stats.total_cost_usd)}</div>
                  <div className="usage-card-label">Est. Cost (USD)</div>
                </div>
              </div>
            </div>

            {/* Today Stats */}
            {stats.today && (
              <div className="config-section">
                <div className="config-section-header">
                  <div className="config-section-icon">
                    <TrendingUp size={16} />
                  </div>
                  <div className="config-section-title">Today</div>
                </div>
                <div className="config-section-body">
                  <div className="today-stats">
                    <div className="today-stat">
                      <span className="today-stat-value">{formatNumber(stats.today.requests)}</span>
                      <span className="today-stat-label">Requests</span>
                    </div>
                    <div className="today-stat">
                      <span className="today-stat-value">{formatNumber(stats.today.tokens)}</span>
                      <span className="today-stat-label">Tokens</span>
                    </div>
                    <div className="today-stat">
                      <span className="today-stat-value">{formatCost(stats.today.cost_usd)}</span>
                      <span className="today-stat-label">Cost</span>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {/* Info */}
            <div className="config-section">
              <div className="config-section-header">
                <div className="config-section-icon">
                  <Activity size={16} />
                </div>
                <div className="config-section-title">About Token Usage</div>
              </div>
              <div className="config-section-body">
                <p style={{ fontSize: '13px', color: 'var(--text-secondary)', lineHeight: 1.6 }}>
                  Token usage is calculated based on the LLM API response. Actual costs may vary slightly depending on the billing method of your LLM provider.
                  Statistics are aggregated from all sessions and agents.
                </p>
              </div>
            </div>
          </>
        )}
      </div>
    </>
  )
}