import { create } from 'zustand'

export type Tab = 'chat' | 'sessions' | 'tools' | 'skills' | 'plugins' | 'usage' | 'agents' | 'settings'
export type Theme = 'dark' | 'light'

const THEME_KEY = 'claw-theme'

function readLocal(key: string, fallback: Theme): Theme {
  if (typeof localStorage === 'undefined') return fallback
  const v = localStorage.getItem(key)
  return v === 'light' || v === 'dark' ? v : fallback
}

interface UiState {
  theme: Theme
  selectedTab: Tab
  mobileDrawerOpen: boolean
  sidebarCollapsed: boolean
  setTheme: (t: Theme) => void
  toggleTheme: () => void
  setSelectedTab: (t: Tab) => void
  setMobileDrawerOpen: (open: boolean) => void
  toggleSidebar: () => void
}

export const useUiStore = create<UiState>((set) => ({
  theme: readLocal(THEME_KEY, 'dark'),
  selectedTab: 'chat',
  mobileDrawerOpen: false,
  sidebarCollapsed: false,
  setTheme: (t) => {
    localStorage.setItem(THEME_KEY, t)
    document.documentElement.setAttribute('data-theme', t)
    set({ theme: t })
  },
  toggleTheme: () => {
    const next: Theme = (localStorage.getItem(THEME_KEY) as Theme) === 'light' ? 'dark' : 'light'
    localStorage.setItem(THEME_KEY, next)
    document.documentElement.setAttribute('data-theme', next)
    set({ theme: next })
  },
  setSelectedTab: (t) => set({ selectedTab: t }),
  setMobileDrawerOpen: (open) => set({ mobileDrawerOpen: open }),
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
}))
