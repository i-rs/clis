import { useMemo } from 'react'
import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import type { Components } from 'react-markdown'
import styles from './MarkdownRenderer.module.css'

interface Props {
  content: string
}

export default function MarkdownRenderer({ content }: Props) {
  const components: Components = useMemo(() => ({
    code: ({ className, children, ...props }) => {
      const isInline = !className
      if (isInline) {
        return (
          <code className={styles.inlineCode} {...props}>
            {children}
          </code>
        )
      }
      return (
        <pre className={styles.codeBlock}>
          <code className={className} {...props}>
            {children}
          </code>
        </pre>
      )
    },
    pre: ({ children }) => <>{children}</>,
    a: ({ href, children }) => (
      <a href={href} target="_blank" rel="noopener noreferrer">
        {children}
      </a>
    ),
    table: ({ children }) => (
      <div className={styles.tableWrapper}>
        <table className={styles.mdTable}>{children}</table>
      </div>
    ),
    blockquote: ({ children }) => (
      <blockquote className={styles.mdBlockquote}>{children}</blockquote>
    ),
    ul: ({ children }) => <ul className={styles.mdList}>{children}</ul>,
    ol: ({ children }) => <ol className={styles.mdList}>{children}</ol>,
    h1: ({ children }) => <h1 className={styles.mdH1}>{children}</h1>,
    h2: ({ children }) => <h2 className={styles.mdH2}>{children}</h2>,
    h3: ({ children }) => <h3 className={styles.mdH3}>{children}</h3>,
    h4: ({ children }) => <h4 className={styles.mdH4}>{children}</h4>,
    h5: ({ children }) => <h5 className={styles.mdH5}>{children}</h5>,
    h6: ({ children }) => <h6 className={styles.mdH6}>{children}</h6>,
  }), [])

  return (
    <div className={styles.content}>
      <ReactMarkdown
        key={content}
        remarkPlugins={[remarkGfm]}
        components={components as any}
      >
        {content}
      </ReactMarkdown>
    </div>
  )
}
