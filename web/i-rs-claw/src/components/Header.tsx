import { useState } from "react";

const NAV_ITEMS = [
  { label: "功能", href: "#features" },
  { label: "各端展示", href: "#platforms" },
  { label: "安装", href: "#install" },
];

export default function Header() {
  const [open, setOpen] = useState(false);

  return (
    <header className="header">
      <a href="#" className="header-logo">i-rs-claw</a>
      <nav className={`header-nav${open ? " open" : ""}`}>
        {NAV_ITEMS.map((item) => (
          <a key={item.href} href={item.href} onClick={() => setOpen(false)}>
            {item.label}
          </a>
        ))}
      </nav>
      <button className="header-toggle" onClick={() => setOpen(!open)}>
        <span /> <span /> <span />
      </button>
    </header>
  );
}
