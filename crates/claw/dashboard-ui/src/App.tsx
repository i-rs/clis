import { useState, useCallback } from 'react'
import { MessageSquareText, History, Settings, Wrench, Puzzle, BookOpen } from 'lucide-react'
import ChatPage from './pages/Chat'
import SessionsPage from './pages/Sessions'
import ConfigPage from './pages/Config'
import ToolsPage from './pages/Tools'
import PluginsPage from './pages/Plugins'
import SkillsPage from './pages/Skills'

type Page = 'chat' | 'sessions' | 'config' | 'tools' | 'plugins' | 'skills'

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

  const navigateTo = useCallback((page: Page) => {
    setCurrentPage(page)
  }, [])

  const refreshSessions = useCallback(() => {
    setSessionRefreshKey((k) => k + 1)
  }, [])

  const renderPage = () => {
    switch (currentPage) {
      case 'chat':
        return <ChatPage key={sessionRefreshKey} onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'sessions':
        return <SessionsPage onNavigate={navigateTo} onSessionChange={refreshSessions} />
      case 'config':
        return <ConfigPage />
      case 'tools':
        return <ToolsPage />
      case 'plugins':
        return <PluginsPage />
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
