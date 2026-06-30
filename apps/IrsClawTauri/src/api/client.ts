import { useBackendStore } from '../store/backend'
import type { ApiResponse } from './types'

export function baseUrl(): string {
  return useBackendStore.getState().baseUrl.replace(/\/$/, '')
}

export function authHeaders(): Record<string, string> {
  const token = useBackendStore.getState().token
  if (!token) return {}
  return { Authorization: `Bearer ${token}` }
}

export async function authFetch(path: string, options?: RequestInit): Promise<Response> {
  return fetch(`${baseUrl()}/api${path}`, {
    ...options,
    headers: { ...authHeaders(), ...(options?.headers || {}) },
  })
}

export async function apiGet<T>(path: string): Promise<ApiResponse<T>> {
  const res = await authFetch(path)
  return res.json()
}

export async function apiJson<T>(path: string, method: string, body?: unknown): Promise<ApiResponse<T>> {
  const res = await authFetch(path, {
    method,
    headers: { 'Content-Type': 'application/json' },
    body: body ? JSON.stringify(body) : undefined,
  })
  return res.json()
}
