import { apiGet, apiJson } from './client'
import type { AgentInfo } from './types'

export const listAgents = () => apiGet<AgentInfo[]>('/agents')
export const getAgentConfig = (id: string) => apiGet<AgentInfo>(`/agents/${encodeURIComponent(id)}`)
export const createAgent = (body: Record<string, unknown>) => apiJson<{ id: string; status: string }>('/agents', 'POST', body)
export const updateAgent = (id: string, body: Record<string, unknown>) => apiJson<{ id: string; status: string }>(`/agents/${encodeURIComponent(id)}`, 'PUT', body)
export const deleteAgent = (id: string) => apiJson<{ id: string; status: string }>(`/agents/${encodeURIComponent(id)}`, 'DELETE')
