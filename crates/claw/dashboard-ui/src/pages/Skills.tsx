import { useState, useEffect } from 'react'
import { BookOpen, ChevronRight, ChevronDown } from 'lucide-react'
import { listSkills, type SkillInfo } from '../api'

export default function SkillsPage() {
  const [skills, setSkills] = useState<SkillInfo[]>([])
  const [loading, setLoading] = useState(true)
  const [expanded, setExpanded] = useState<Set<string>>(new Set())

  useEffect(() => {
    listSkills().then((resp) => {
      if (resp.success && resp.data) {
        setSkills(resp.data)
      }
      setLoading(false)
    }).catch(() => setLoading(false))
  }, [])

  const toggleExpand = (name: string) => {
    const next = new Set(expanded)
    if (next.has(name)) {
      next.delete(name)
    } else {
      next.add(name)
    }
    setExpanded(next)
  }

  return (
    <>
      <div className="page-header">
        <h2>Skills ({skills.length})</h2>
      </div>
      <div className="page-body">
        {loading ? (
          <div className="loading">
            <div className="loading-spinner" />
            Loading skills...
          </div>
        ) : skills.length === 0 ? (
          <div className="empty-state">
            <BookOpen size={40} className="empty-state-icon" />
            <p>No skills found. Add <code>.md</code> skill files to <code>~/.i-rs-claw/claw/skills/</code></p>
          </div>
        ) : (
          <div className="skill-list">
            {skills.map((skill) => (
              <div key={skill.name} className="skill-card">
                <div
                  className="skill-header"
                  onClick={() => toggleExpand(skill.name)}
                  role="button"
                  tabIndex={0}
                  onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') toggleExpand(skill.name) }}
                >
                  <span className="skill-name">
                      {skill.name}
                      {skill.description && skill.description !== skill.name ? (
                        <span className="skill-desc"> — {skill.description}</span>
                      ) : null}
                    </span>
                  <span className={`skill-toggle ${expanded.has(skill.name) ? 'expanded' : ''}`}>
                    {expanded.has(skill.name) ? <ChevronDown size={12} /> : <ChevronRight size={12} />}
                  </span>
                </div>
                {expanded.has(skill.name) && (
                  <div className="skill-content">
                    <pre className="skill-content-pre">{skill.content}</pre>
                  </div>
                )}
              </div>
            ))}
          </div>
        )}
      </div>
    </>
  )
}
