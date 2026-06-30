import type { ReactNode } from 'react'
import Sidebar from './Sidebar'
import '../../styles/responsive.css'

interface Props { children: ReactNode }

export default function DesktopShell({ children }: Props) {
  return (
    <div className="app-shell">
      <Sidebar />
      <main className="app-detail">{children}</main>
    </div>
  )
}
