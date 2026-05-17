import { useState, useCallback, useEffect } from 'react'
import { MessageSquareText, History, Settings, Wrench, Puzzle, BookOpen, Bot, Users } from 'lucide-react'
import { listAgents, type AgentInfo } from './api'
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

export default function App() {
  const [currentPage, setCurrentPage] = useState<Page>('chat')
  const [sessionRefreshKey, setSessionRefreshKey] = useState(0)
  const [selectedAgent, setSelectedAgent] = useState('default')
  const [agents, setAgents] = useState<AgentInfo[]>([])
  const [agentRefreshKey, setAgentRefreshKey] = useState(0)

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

  const navigateTo = useCallback((page: Page) => {
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
        return <ChatPage key={sessionRefreshKey} selectedAgent={selectedAgent} onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'sessions':
        return <SessionsPage selectedAgent={selectedAgent} onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'config':
        return <ConfigPage selectedAgent={selectedAgent} onAgentsChange={refreshAgents} />
      case 'tools':
        return <ToolsPage />
      case 'plugins':
        return <PluginsPage />
      case 'agents':
        return <AgentsPage onAgentsChange={refreshAgents} onNavigate={navigateTo} />
      case 'skills':
        return <SkillsPage />
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
      </aside>
      <main className="main-content">
        {renderPage()}
      </main>
    </div>
  )
}
