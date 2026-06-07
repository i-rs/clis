import { useState, useEffect } from 'react'
import { Search, Database, Activity, Heart, DollarSign, Calendar, Users, Car, Home } from 'lucide-react'
import { listTools, type ToolSchema } from '../api'

interface ToolCategory {
  name: string
  icon: React.ReactNode
  tools: string[]
}

const I_RS_TOOLS: ToolCategory[] = [
  {
    name: 'Health & Body',
    icon: <Heart size={16} />,
    tools: ['weight', 'height', 'sleep', 'water', 'step', 'dose', 'cycle', 'sit', 'allergy', 'cal', 'fast', 'exercise', 'run', 'cycling', 'mood', 'habit'],
  },
  {
    name: 'Finance',
    icon: <DollarSign size={16} />,
    tools: ['ledger', 'recur', 'budget', 'invest', 'debt', 'invoice', 'tax', 'goal', 'sub'],
  },
  {
    name: 'Personal Data',
    icon: <Database size={16} />,
    tools: ['bookmark', 'note', 'password', 'kv', 'keys', 'want', 'gift', 'movie', 'podcast', 'contact', 'read', 'quote', 'snippet', 'vocab', 'time', 'spark', 'pig', 'meal', 'grocery', 'tick'],
  },
  {
    name: 'Projects & Work',
    icon: <Activity size={16} />,
    tools: ['project', 'article', 'deploy', 'vision', 'server'],
  },
  {
    name: 'Events & Reminders',
    icon: <Calendar size={16} />,
    tools: ['remind', 'birthday', 'event', 'domain', 'bestby', 'todo'],
  },
  {
    name: 'People & Social',
    icon: <Users size={16} />,
    tools: ['contact'],
  },
  {
    name: 'Vehicle',
    icon: <Car size={16} />,
    tools: ['car'],
  },
  {
    name: 'Home & Pets',
    icon: <Home size={16} />,
    tools: ['sheet', 'toothbrush', 'towel', 'bed', 'ac', 'filter', 'purify', 'appliance', 'plant', 'feedpet', 'petbath', 'walkdog', 'aqua'],
  },
]

export default function DataPage() {
  const [tools, setTools] = useState<ToolSchema[]>([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null)
  const [selectedTool, setSelectedTool] = useState<string | null>(null)

  useEffect(() => {
    listTools().then((res) => {
      if (res.success && res.data) setTools(res.data)
      setLoading(false)
    }).catch(() => setLoading(false))
  }, [])

  const iRsTools = tools.filter((t) =>
    t.function?.name?.startsWith('i_rs_') || t.function?.name?.startsWith('i-rs')
  )

  const categories = [...new Set(I_RS_TOOLS.map((c) => c.name))]

  const filteredCategories = selectedCategory
    ? I_RS_TOOLS.filter((c) => c.name === selectedCategory)
    : I_RS_TOOLS.filter((c) =>
        search ? c.tools.some((t) => t.includes(search.toLowerCase())) : true
      )

  return (
    <div className="page-container">
      <div className="page-header">
        <div className="page-header-left">
          <div className="page-header-icon">
            <Database size={16} />
          </div>
          <h2>Data Explorer</h2>
          <span className="badge badge-info">{iRsTools.length} tools</span>
        </div>
      </div>

      <div className="page-toolbar">
        <div className="search-box">
          <Search size={14} />
          <input
            type="text"
            placeholder="Search tools..."
            value={search}
            onChange={(e) => { setSearch(e.target.value); setSelectedCategory(null) }}
          />
        </div>
        <div className="category-filters">
          <button
            className={`filter-chip ${!selectedCategory ? 'active' : ''}`}
            onClick={() => { setSelectedCategory(null); setSearch('') }}
          >
            All
          </button>
          {categories.map((cat) => (
            <button
              key={cat}
              className={`filter-chip ${selectedCategory === cat ? 'active' : ''}`}
              onClick={() => setSelectedCategory(selectedCategory === cat ? null : cat)}
            >
              {I_RS_TOOLS.find((c) => c.name === cat)?.icon}
              {cat}
            </button>
          ))}
        </div>
      </div>

      {loading ? (
        <div className="page-loading">
          <div className="typing-dots"><span></span><span></span><span></span></div>
          Loading tools...
        </div>
      ) : (
        <div className="data-grid">
          {filteredCategories.map((cat) => (
            <div key={cat.name} className="data-category">
              <div className="data-category-header">
                <span className="data-category-icon">{cat.icon}</span>
                <h3>{cat.name}</h3>
                <span className="badge badge-secondary">{cat.tools.length}</span>
              </div>
              <div className="data-category-tools">
                {cat.tools
                  .filter((t) => !search || t.includes(search.toLowerCase()))
                  .map((tool) => (
                    <button
                      key={tool}
                      className={`data-tool-card ${selectedTool === tool ? 'selected' : ''}`}
                      onClick={() => setSelectedTool(selectedTool === tool ? null : tool)}
                    >
                      <span className="data-tool-icon">
                        <Activity size={14} />
                      </span>
                      <div className="data-tool-info">
                        <span className="data-tool-name">i-rs {tool}</span>
                        <span className="data-tool-hint">View data</span>
                      </div>
                    </button>
                  ))}
              </div>
            </div>
          ))}
        </div>
      )}

      {selectedTool && (
        <div className="data-detail">
          <div className="data-detail-header">
            <h3>i-rs {selectedTool}</h3>
            <button className="btn btn-ghost btn-sm" onClick={() => setSelectedTool(null)}>Close</button>
          </div>
          <div className="data-detail-body">
            <div className="data-placeholder">
              <Database size={48} />
              <h4>Coming Soon</h4>
              <p>
                Data display for <strong>i-rs {selectedTool}</strong> will be available
                once the <code>/api/data/{'{'}tool{'}'}</code> endpoint is implemented.
              </p>
              <p className="text-muted">
                Use the chat interface to interact with your data. Try asking
                Claw to show your data, e.g., "Show my weight records" or "How many steps
                did I take this week?"
              </p>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
