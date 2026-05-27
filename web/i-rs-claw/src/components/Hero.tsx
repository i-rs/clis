export default function Hero() {
  return (
    <section className="hero-section">
      <div className="hero-badge">v0.0.1 — TUI AI Assistant</div>
      <h1 className="hero-title">
        你的终端<br />智能助理
      </h1>
      <p className="hero-desc">
        i-rs-claw 是一个终端 AI 助理，深度集成 70+ 个人数据管理工具，
        支持多模型、多 Agent、MCP 协议扩展。
      </p>
      <div className="hero-actions">
        <a href="#install" className="btn btn-primary">立即安装</a>
        <a href="https://github.com/i-rs/clis" className="btn btn-outline" target="_blank">GitHub</a>
      </div>
      <div className="hero-terminal">
        <div className="terminal-dot" />
        <span className="terminal-text">i-rs-claw chat "帮我查一下这个月的跑步记录"</span>
      </div>
    </section>
  );
}
