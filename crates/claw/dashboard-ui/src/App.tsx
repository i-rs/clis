import { useState, useCallback, useEffect } from 'react'
import { MessageSquareText, History, Settings, Wrench, Puzzle, BookOpen, Bot, Users, Lock, Sun, Moon } from 'lucide-react'
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
  { id: 'config', label: 'Config', icon: <Settings size={18} /> },
  { id: 'tools', label: 'Tools', icon: <Wrench size={18} /> },
  { id: 'plugins', label: 'Plugins', icon: <Puzzle size={18} /> },
  { id: 'skills', label: 'Skills', icon: <BookOpen size={18} /> },
]

function TokenPrompt({ onSubmit }: { onSubmit: (token: string) => void }) {
  const [value, setValue] = useState('')

  return (
    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100vh', background: '#0f0f23' }}>
      <div style={{ background: '#1a1a2e', borderRadius: 12, padding: '40px 32px', width: 380, textAlign: 'center' }}>
        <Lock size={40} color="#6366f1" style={{ marginBottom: 16 }} />
        <h2 style={{ color: '#e2e8f0', margin: '0 0 8px' }}>Dashboard 需要认证</h2>
        <p style={{ color: '#94a3b8', fontSize: 14, margin: '0 0 24px' }}>
          输入启动时打印的 token，或在 config.toml 中配置 <code style={{ background: '#334155', padding: '2px 6px', borderRadius: 4 }}>dashboard.auth_token</code>
        </p>
        <input
          type="password"
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={(e) => { if (e.key === 'Enter' && value.trim()) onSubmit(value.trim()) }}
          placeholder="粘贴 token..."
          autoFocus
          style={{
            width: '100%', padding: '10px 14px', borderRadius: 8, border: '1px solid #334155',
            background: '#0f172a', color: '#e2e8f0', fontSize: 14, outline: 'none', boxSizing: 'border-box',
          }}
        />
        <button
          onClick={() => { if (value.trim()) onSubmit(value.trim()) }}
          disabled={!value.trim()}
          style={{
            marginTop: 16, width: '100%', padding: '10px 0', borderRadius: 8, border: 'none',
            background: value.trim() ? '#6366f1' : '#334155', color: '#fff', fontSize: 14,
            fontWeight: 600, cursor: value.trim() ? 'pointer' : 'not-allowed',
          }}
        >
          确认
        </button>
      </div>
    </div>
  )
}

export default function App() {
  const [authenticated, setAuthenticated] = useState(hasToken())
  const [currentPage, setCurrentPage] = useState<Page>('chat')
  const [pageKey, setPageKey] = useState(0)
  const [sessionRefreshKey, setSessionRefreshKey] = useState(0)
  const [selectedAgent, setSelectedAgent] = useState('default')
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [agentRefreshKey, setAgentRefreshKey] = useState(0)
  const [theme, setTheme] = useState(() => {
    const saved = localStorage.getItem('claw-theme')
    return saved || 'dark'
  })

  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
    localStorage.setItem('claw-theme', theme)
  }, [theme])

  const toggleTheme = () => setTheme(t => t === 'dark' ? 'light' : 'dark')

  // Load agents list
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
    setSessionRefreshKey((k) => k + 1)
  }, [])

  const refreshAgents = useCallback(() => {
    setAgentRefreshKey((k) => k + 1)
  }, [])

  const handleSwitchAgent = (id: string) => {
    setSelectedAgent(id)
    refreshSessions()
    setCurrentPage('chat')
  }

  const renderPage = () => {
    switch (currentPage) {
      case 'chat':
        return <ChatPage key={`${sessionRefreshKey}-${pageKey}`} selectedAgent={selectedAgent} onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'sessions':
        return <SessionsPage key={pageKey} selectedAgent={selectedAgent} onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'config':
        return <ConfigPage key={pageKey} selectedAgent={selectedAgent} onAgentsChange={refreshAgents} />
      case 'tools':
        return <ToolsPage key={pageKey} />
      case 'plugins':
        return <PluginsPage key={pageKey} />
      case 'agents':
        return <AgentsPage key={pageKey} onAgentsChange={refreshAgents} onNavigate={navigateTo} />
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
        <div className="agent-switcher">
          <div className="agent-switcher-select">
            <Bot size={14} className="agent-switcher-icon" />
            <select
              value={selectedAgent}
              onChange={(e) => handleSwitchAgent(e.target.value)}
              className="agent-switcher-dropdown"
            >
              {agents.map((a) => (
                <option key={a.id} value={a.id}>{a.id}</option>
              ))}
            </select>
          </div>
          <button
            className="agent-switcher-btn"
            onClick={() => navigateTo('agents')}
            title="Manage agents"
          >
            <Users size={14} />
          </button>
        </div>

        <nav className="sidebar-nav">
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
        </nav>

        <div className="sidebar-footer">
          <button className="theme-toggle" onClick={toggleTheme} title="切换主题">
            {theme === 'dark' ? <Sun size={16} /> : <Moon size={16} />}
          </button>
        </div>
      </aside>
      <main className="main-content">
        {renderPage()}
      </main>
    </div>
  )
}
