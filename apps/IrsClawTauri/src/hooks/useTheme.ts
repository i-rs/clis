import { useEffect } from 'react'
import { useUiStore } from '../store/ui'

export function useTheme() {
  const theme = useUiStore((s) => s.theme)
  const toggleTheme = useUiStore((s) => s.toggleTheme)
  useEffect(() => { document.documentElement.setAttribute('data-theme', theme) }, [theme])
  return { theme, toggleTheme }
}
