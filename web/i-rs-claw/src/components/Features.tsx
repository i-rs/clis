const FEATURES = [
  {
    title: "AI 对话",
    desc: "自然语言与你的数据交互，支持多轮上下文理解，自动选择工具完成任务。",
  },
  {
    title: "70+ 工具集成",
    desc: "体重、记账、待办、习惯、阅读…覆盖个人数据管理的方方面面。",
  },
  {
    title: "多模型支持",
    desc: "OpenAI、Anthropic、Ollama、智谱…自由切换，也可按 Agent 独立配置。",
  },
  {
    title: "MCP 协议扩展",
    desc: "通过 MCP 协议接入浏览器控制、文件系统、网页搜索等外部工具。",
  },
  {
    title: "多终端适配",
    desc: "TUI、Web Dashboard、macOS、iPad、iOS、微信小程序，数据统一。",
  },
  {
    title: "ReAct 智能循环",
    desc: "思考→调用工具→观察结果→继续推理，直到完成你的指令。",
  },
];

export default function Features() {
  return (
    <section id="features">
      <div className="container">
        <p className="section-label">Features</p>
        <h2 className="section-title">核心能力</h2>
        <div className="features-grid">
          {FEATURES.map((f) => (
            <div key={f.title} className="feature-card">
              <h3>{f.title}</h3>
              <p>{f.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
