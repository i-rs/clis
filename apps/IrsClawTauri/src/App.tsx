import { useState } from 'react'
import { Route, Switch } from 'wouter'
import { Wrench, BookOpen, Puzzle } from 'lucide-react'
import { useIsDesktop } from './hooks/useMediaQuery'
import { useTheme } from './hooks/useTheme'
import { useBackendHealth } from './hooks/useBackendHealth'
import { useBackendStore } from './store/backend'
import DesktopShell from './components/shell/DesktopShell'
import MobileShell from './components/shell/MobileShell'
import ConnectScreen from './components/shell/ConnectScreen'
import { ToastContainer } from './components/common/Toast'
import ChatPage from './pages/Chat'
import SessionsPage from './pages/Sessions'
import AgentsPage from './pages/Agents'
import UsagePage from './pages/Usage'
import SettingsPage from './pages/Settings'
import PlaceholderPage from './pages/Placeholder'

export default function App() {
  useTheme()
  useBackendHealth()
  const isDesktop = useIsDesktop()
  const connectionState = useBackendStore((s) => s.connectionState)
  const [selectedAgent] = useState('default')

  if (connectionState !== 'connected') {
    return (<><ConnectScreen /><ToastContainer /></>)
  }

  const Shell = isDesktop ? DesktopShell : MobileShell

  return (
    <>
      <Shell>
        <Switch>
          <Route path="/"><ChatPage selectedAgent={selectedAgent} /></Route>
          <Route path="/sessions"><SessionsPage /></Route>
          <Route path="/agents"><AgentsPage /></Route>
          <Route path="/usage"><UsagePage /></Route>
          <Route path="/settings"><SettingsPage /></Route>
          <Route path="/tools"><PlaceholderPage icon={<Wrench size={28} />} title="Tools" /></Route>
          <Route path="/skills"><PlaceholderPage icon={<BookOpen size={28} />} title="Skills" /></Route>
          <Route path="/plugins"><PlaceholderPage icon={<Puzzle size={28} />} title="Plugins" /></Route>
        </Switch>
      </Shell>
      <ToastContainer />
    </>
  )
}
