export default function Footer() {
  return (
    <footer className="footer">
      <div className="footer-inner">
        <div className="footer-brand">
          <strong>i-rs-claw</strong>
          <p>个人数据智能助理</p>
        </div>
        <div className="footer-links">
          <a href="https://github.com/i-rs/clis" target="_blank">GitHub</a>
          <a href="https://github.com/i-rs/clis/issues" target="_blank">Issues</a>
          <a href="/docs" target="_blank">文档</a>
        </div>
        <p className="footer-copy">MIT OR Apache-2.0</p>
      </div>
    </footer>
  );
}
