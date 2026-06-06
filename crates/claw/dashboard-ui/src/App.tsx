import { useState, useCallback, useEffect, useRef } from 'react'
import { Route, Switch, useLocation } from 'wouter'
import { MessageSquareText, History, Settings, Wrench, Puzzle, BookOpen, Bot, Lock, Sun, Moon, ChevronDown, BarChart3 } from 'lucide-react'
import { listAgents, type AgentInfo, hasToken, setToken } from './api'
import ChatPage from './pages/Chat'
import SessionsPage from './pages/Sessions'
import ConfigPage from './pages/Config'
import ToolsPage from './pages/Tools'
import PluginsPage from './pages/Plugins'
import SkillsPage from './pages/Skills'
import AgentsPage from './pages/Agents'
import UsagePage from './pages/Usage'

type Page = 'chat' | 'sessions' | 'config' | 'tools' | 'plugins' | 'skills' | 'agents' | 'usage'

const NAV_ITEMS: { id: Page; label: string; icon: React.ReactNode }[] = [
  { id: 'chat', label: 'Chat', icon: <MessageSquareText size={18} /> },
  { id: 'sessions', label: 'Sessions', icon: <History size={18} /> },
  { id: 'agents', label: 'Agents', icon: <Bot size={18} /> },
  { id: 'usage', label: 'Usage', icon: <BarChart3 size={18} /> },
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
  const [location, setLocation] = useLocation()
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
        const userAgents = resp.data.filter((a) => !a.is_sub_agent)
        setAgents(userAgents)
        const exists = userAgents.some((a) => a.id === selectedAgent)
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
    setLocation(page === 'chat' ? '/' : `/${page}`)
  }, [setLocation])

  const refreshAgents = useCallback(() => {
    setAgentRefreshKey((k) => k + 1)
  }, [])

  const handleSwitchAgent = (id: string) => {
    setSelectedAgent(id)
    setShowAgentDropdown(false)
    setLocation('/')
  }

  const currentPage = location === '/' ? 'chat' : (location.slice(1) as Page) || 'chat'

  return (
    <div className="app-layout">
      <aside className="sidebar">
        <div className="sidebar-header">
          <h1>i-rs-claw</h1>
          <div className="subtitle">AI Personal Assistant</div>
        </div>

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
        <Switch>
          <Route path="/">
            {() => <ChatPage selectedAgent={selectedAgent} onNavigate={navigateTo} />}
          </Route>
          <Route path="/sessions">
            {() => <SessionsPage selectedAgent={selectedAgent} onNavigate={navigateTo} />}
          </Route>
          <Route path="/agents">
            {() => <AgentsPage onAgentsChange={refreshAgents} />}
          </Route>
          <Route path="/usage">
            {() => <UsagePage />}
          </Route>
          <Route path="/config">
            {() => <ConfigPage selectedAgent={selectedAgent} onAgentsChange={refreshAgents} />}
          </Route>
          <Route path="/tools">
            {() => <ToolsPage />}
          </Route>
          <Route path="/plugins">
            {() => <PluginsPage />}
          </Route>
          <Route path="/skills">
            {() => <SkillsPage />}
          </Route>
        </Switch>
      </main>
    </div>
  )
}
