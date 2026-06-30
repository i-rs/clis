import { describe, it, expect, beforeEach } from 'vitest'
import { useBackendStore } from './backend'

describe('backendStore', () => {
  beforeEach(() => {
    localStorage.clear()
    useBackendStore.setState({ baseUrl: 'http://localhost:3000', token: '', connectionState: 'disconnected', errorMessage: null })
  })

  it('persists baseUrl and token to localStorage', () => {
    useBackendStore.getState().setBaseUrl('http://192.168.1.5:3000')
    useBackendStore.getState().setToken('abc123')
    expect(localStorage.getItem('claw-backend-url')).toBe('http://192.168.1.5:3000')
    expect(localStorage.getItem('claw-dashboard-token')).toBe('abc123')
  })

  it('transitions connection state', () => {
    useBackendStore.getState().setConnecting()
    expect(useBackendStore.getState().connectionState).toBe('connecting')
    useBackendStore.getState().setConnected()
    expect(useBackendStore.getState().connectionState).toBe('connected')
    useBackendStore.getState().setFailed('boom')
    expect(useBackendStore.getState().connectionState).toBe('failed')
    expect(useBackendStore.getState().errorMessage).toBe('boom')
  })
})
