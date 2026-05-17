import ReactMarkdown from 'react-markdown'
import remarkGfm from 'remark-gfm'
import type { Components } from 'react-markdown'

interface Props {
  content: string
}

const components: Components = {
  code: ({ className, children, ...props }) => {
    const isInline = !className
    if (isInline) {
      return (
        <code className="inline-code" {...props}>
          {children}
        </code>
      )
    }
    return (
      <pre className="code-block">
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
    <div className="table-wrapper">
      <table className="md-table">{children}</table>
    </div>
  ),
  blockquote: ({ children }) => (
    <blockquote className="md-blockquote">{children}</blockquote>
  ),
  ul: ({ children }) => <ul className="md-list">{children}</ul>,
  ol: ({ children }) => <ol className="md-list">{children}</ol>,
  h1: ({ children }) => <h1 className="md-h1">{children}</h1>,
  h2: ({ children }) => <h2 className="md-h2">{children}</h2>,
  h3: ({ children }) => <h3 className="md-h3">{children}</h3>,
}

export default function MarkdownRenderer({ content }: Props) {
  return (
    <ReactMarkdown
      remarkPlugins={[remarkGfm]}
      components={components}
    >
      {content}
    </ReactMarkdown>
  )
}
