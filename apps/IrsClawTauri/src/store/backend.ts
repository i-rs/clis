import { create } from 'zustand'

export type ConnectionState = 'disconnected' | 'connecting' | 'connected' | 'failed'

const URL_KEY = 'claw-backend-url'
const TOKEN_KEY = 'claw-dashboard-token'

function readLocal(key: string, fallback: string): string {
  if (typeof localStorage === 'undefined') return fallback
  return localStorage.getItem(key) || fallback
}

interface BackendState {
  baseUrl: string
  token: string
  connectionState: ConnectionState
  errorMessage: string | null
  setBaseUrl: (url: string) => void
  setToken: (token: string) => void
  setConnecting: () => void
  setConnected: () => void
  setFailed: (msg: string) => void
  setDisconnected: () => void
}

export const useBackendStore = create<BackendState>((set) => ({
  baseUrl: readLocal(URL_KEY, 'http://localhost:3000'),
  token: readLocal(TOKEN_KEY, ''),
  connectionState: 'disconnected',
  errorMessage: null,
  setBaseUrl: (url) => { localStorage.setItem(URL_KEY, url); set({ baseUrl: url }) },
  setToken: (token) => { localStorage.setItem(TOKEN_KEY, token); set({ token }) },
  setConnecting: () => set({ connectionState: 'connecting', errorMessage: null }),
  setConnected: () => set({ connectionState: 'connected', errorMessage: null }),
  setFailed: (msg) => set({ connectionState: 'failed', errorMessage: msg }),
  setDisconnected: () => set({ connectionState: 'disconnected', errorMessage: null }),
}))
