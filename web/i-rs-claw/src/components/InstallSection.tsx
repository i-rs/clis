const INSTALL_STEPS = [
  {
    title: "macOS / Linux",
    cmd: "cargo install i-rs-claw",
    desc: "通过 Cargo 安装，需要 Rust 工具链。",
  },
  {
    title: "Homebrew",
    cmd: "brew install i-rs/tap/i-rs-claw",
    desc: "macOS 用户推荐使用 Homebrew。",
  },
  {
    title: "从源码构建",
    cmd: "git clone https://github.com/i-rs/clis.git\ncd clis && cargo build -p i-rs-claw",
    desc: "获取最新开发版本。",
  },
  {
    title: "客户端下载",
    cmd: "macOS / iPad / iOS: App Store 搜索 i-rs-claw\n微信小程序: 微信内搜索「i-rs-claw」",
    desc: "移动端直接在各平台应用商店下载。",
  },
];

export default function InstallSection() {
  return (
    <section id="install">
      <div className="container">
        <p className="section-label">Install</p>
        <h2 className="section-title">安装与下载</h2>
        <div className="install-grid">
          {INSTALL_STEPS.map((item) => (
            <div key={item.title} className="install-card">
              <h3>{item.title}</h3>
              <pre><code>{item.cmd}</code></pre>
              <p>{item.desc}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
