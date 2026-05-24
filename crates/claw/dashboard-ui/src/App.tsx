import { useState, useCallback, useEffect, useRef } from 'react'
import { MessageSquareText, History, Settings, Wrench, Puzzle, BookOpen, Bot, Lock, Sun, Moon, ChevronDown } from 'lucide-react'
import { listAgents, type AgentInfo, hasToken, setToken } from './api'
import ChatPage from './pages/Chat'
import SessionsPage from './pages/Sessions'
import ConfigPage from './pages/Config'
import ToolsPage from './pages/Tools'
import PluginsPage from './pages/Plugins'
import SkillsPage from './pages/Skills'
import AgentsPage from './pages/Agents'

type Page = 'chat' | 'sessions' | 'config' | 'tools' | 'plugins' | 'skills' | 'agents'

const NAV_ITEMS: { id: Page; label: string; icon: React.ReactNode }[] = [
  { id: 'chat', label: 'Chat', icon: <MessageSquareText size={18} /> },
  { id: 'sessions', label: 'Sessions', icon: <History size={18} /> },
  { id: 'agents', label: 'Agents', icon: <Bot size={18} /> },
  { id: 'config', label: 'Config', icon: <Settings size={18} /> },
  { id: 'tools', label: 'Tools', icon: <Wrench size={18} /> },
  { id: 'plugins', label: 'Plugins', icon: <Puzzle size={18} /> },
  { id: 'skills', label: 'Skills', icon: <BookOpen size={18} /> },
]

function TokenPrompt({ onSubmit }: { onSubmit: (token: string) => void }) {
  const [value, setValue] = useState('')

  return (
    <div className="token-prompt">
      <div className="token-prompt-card">
        <div className="token-prompt-icon">
          <Lock size={28} />
        </div>
        <h2>Authentication Required</h2>
        <p>
          Enter the token printed at startup, or configure <code>dashboard.auth_token</code> in config.toml
        </p>
        <input
          type="password"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter' && value.trim()) onSubmit(value.trim()) }}
          placeholder="Paste token..."
          autoFocus
        />
        <button
          className="btn btn-primary"
          onClick={() => { if (value.trim()) onSubmit(value.trim()) }}
          disabled={!value.trim()}
        >
          Confirm
        </button>
      </div>
    </div>
  )
}

export default function App() {
  const [authenticated, setAuthenticated] = useState(hasToken())
  const [currentPage, setCurrentPage] = useState<Page>('chat')
  const [pageKey, setPageKey] = useState(0)
  const [selectedAgent, setSelectedAgent] = useState('default')
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [agentRefreshKey, setAgentRefreshKey] = useState(0)
  const [theme, setTheme] = useState(() => {
    const saved = localStorage.getItem('claw-theme')
    return saved || 'dark'
  })
  const [showAgentDropdown, setShowAgentDropdown] = useState(false)
  const dropdownRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
    localStorage.setItem('claw-theme', theme)
  }, [theme])

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(e.target as Node)) {
        setShowAgentDropdown(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => document.removeEventListener('mousedown', handleClickOutside)
  }, [])

  const toggleTheme = () => setTheme(t => t === 'dark' ? 'light' : 'dark')

  useEffect(() => {
    listAgents().then((resp) => {
      if (resp.success && resp.data) {
        setAgents(resp.data)
        const exists = resp.data.some((a) => a.id === selectedAgent)
        if (!exists) setSelectedAgent('default')
      }
    })
  }, [agentRefreshKey])

  const handleTokenSubmit = useCallback((token: string) => {
    setToken(token)
    setAuthenticated(true)
    window.location.reload()
  }, [])

  if (!authenticated) {
    return <TokenPrompt onSubmit={handleTokenSubmit} />
  }

  const navigateTo = useCallback((page: Page) => {
    setPageKey(k => k + 1)
    setCurrentPage(page)
  }, [])

  const refreshSessions = useCallback(() => {
    // Session list is refreshed on mount and agent switch; no remount needed
  }, [])

  const refreshAgents = useCallback(() => {
    setAgentRefreshKey((k) => k + 1)
  }, [])

  const handleSwitchAgent = (id: string) => {
    setSelectedAgent(id)
    setShowAgentDropdown(false)
    refreshSessions()
    setCurrentPage('chat')
  }

  const renderPage = () => {
    switch (currentPage) {
      case 'chat':
        return <ChatPage key={pageKey} selectedAgent={selectedAgent} onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'sessions':
        return <SessionsPage key={pageKey} selectedAgent={selectedAgent} onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'config':
        return <ConfigPage key={pageKey} selectedAgent={selectedAgent} onAgentsChange={refreshAgents} />
      case 'tools':
        return <ToolsPage key={pageKey} />
      case 'plugins':
        return <PluginsPage key={pageKey} />
      case 'agents':
        return <AgentsPage key={pageKey} onAgentsChange={refreshAgents} />
      case 'skills':
        return <SkillsPage key={pageKey} />
    }
  }

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1>i-rs-claw</h1>
          <div className="subtitle">AI Personal Assistant</div>
        </div>

        {/* Agent Switcher */}
        <div className="agent-switcher" ref={dropdownRef}>
          <div className="agent-switcher-container" onClick={() => setShowAgentDropdown(!showAgentDropdown)}>
            <div className="agent-switcher-icon">
              <Bot size={14} />
            </div>
            <div className="agent-switcher-info">
              <div className="agent-switcher-label">Agent</div>
              <div className="agent-switcher-name">{selectedAgent}</div>
            </div>
            <ChevronDown size={14} style={{ color: 'var(--text-muted)', transition: 'transform 0.15s', transform: showAgentDropdown ? 'rotate(180deg)' : 'rotate(0deg)' }} />
          </div>
          {showAgentDropdown && (
            <div className="agent-switcher-dropdown">
              {agents.map((a) => (
                <div
                  key={a.id}
                  className={`agent-dropdown-item ${a.id === selectedAgent ? 'active' : ''}`}
                  onClick={() => handleSwitchAgent(a.id)}
                >
                  <Bot size={14} />
                  {a.id}
                </div>
              ))}
            </div>
          )}
        </div>

        <nav className="sidebar-nav">
          <div className="nav-section">
            <div className="nav-section-label">Menu</div>
            {NAV_ITEMS.map((item) => (
              <button
                key={item.id}
                className={`nav-item${currentPage === item.id ? ' active' : ''}`}
                onClick={() => navigateTo(item.id)}
              >
                <span className="nav-icon">{item.icon}</span>
                {item.label}
              </button>
            ))}
          </div>
        </nav>

        <div className="sidebar-footer">
          <div className="sidebar-footer-content">
            <button className="theme-toggle" onClick={toggleTheme} title="Toggle theme">
              {theme === 'dark' ? <Sun size={16} /> : <Moon size={16} />}
            </button>
          </div>
        </div>
      </aside>
      <main className="main-content">
        {renderPage()}
      </main>
    </div>
  )
}
