import { apiGet } from './client'
import type { StatsResponse } from './types'

export const getStats = (period: string = 'all') =>
  apiGet<StatsResponse>(`/stats?period=${encodeURIComponent(period)}`)
