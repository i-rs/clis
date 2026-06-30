import type { ReactNode } from 'react'
import EmptyState from '../components/common/EmptyState'

export default function PlaceholderPage({ icon, title }: { icon: ReactNode; title: string }) {
  return <EmptyState icon={icon} title={title} description="This page is coming soon. Use the dashboard-ui web interface for full management in the meantime." />
}
