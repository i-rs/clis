import { useState, useCallback, useEffect, useRef } from "react";

interface Screenshot {
  id: number;
  label: string;
  gradient: string;
}

interface PlatformProps {
  name: string;
  mockup: string;
  description: string;
  screenshots: Screenshot[];
}

function Carousel({ items }: { items: Screenshot[] }) {
  const [idx, setIdx] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval>>(undefined);

  const go = useCallback((i: number) => {
    setIdx(((i % items.length) + items.length) % items.length);
  }, [items.length]);

  useEffect(() => {
    timerRef.current = setInterval(() => go(idx + 1), 4000);
    return () => clearInterval(timerRef.current);
  }, [idx, go]);

  const item = items[idx];

  return (
    <div className="carousel">
      <div
        className="carousel-image"
        style={{ background: item.gradient }}
      >
        <span className="carousel-placeholder">{item.label}</span>
      </div>
      <div className="carousel-controls">
        <button onClick={() => go(idx - 1)}>‹</button>
        <div className="carousel-dots">
          {items.map((_, i) => (
            <button
              key={i}
              className={`dot${i === idx ? " active" : ""}`}
              onClick={() => go(i)}
            />
          ))}
        </div>
        <button onClick={() => go(idx + 1)}>›</button>
      </div>
    </div>
  );
}

const PLATFORMS: PlatformProps[] = [
  {
    name: "TUI 终端",
    mockup: "terminal",
    description: "全键盘操作的终端界面，支持分屏、主题定制、输入补全。",
    screenshots: [
      { id: 1, label: "对话界面", gradient: "linear-gradient(135deg, #1e1b4b, #312e81)" },
      { id: 2, label: "工具调用", gradient: "linear-gradient(135deg, #0f172a, #1e293b)" },
      { id: 3, label: "会话管理", gradient: "linear-gradient(135deg, #020617, #0f172a)" },
    ],
  },
  {
    name: "Dashboard UI",
    mockup: "browser",
    description: "Web 仪表盘，嵌入 TUI 的可视化管理界面。",
    screenshots: [
      { id: 1, label: "数据概览", gradient: "linear-gradient(135deg, #0f766e, #14b8a6)" },
      { id: 2, label: "图表展示", gradient: "linear-gradient(135deg, #065f46, #059669)" },
    ],
  },
  {
    name: "macOS",
    mockup: "macbook",
    description: "原生 macOS 应用，支持菜单栏快速访问、系统通知集成。",
    screenshots: [
      { id: 1, label: "主界面", gradient: "linear-gradient(135deg, #4c1d95, #7c3aed)" },
      { id: 2, label: "菜单栏", gradient: "linear-gradient(135deg, #3b0764, #6b21a8)" },
    ],
  },
  {
    name: "iPad",
    mockup: "ipad",
    description: "针对 iPad 大屏优化的自适应布局，支持 Split View 多任务。",
    screenshots: [
      { id: 1, label: "横屏模式", gradient: "linear-gradient(135deg, #1e40af, #3b82f6)" },
      { id: 2, label: "分屏协作", gradient: "linear-gradient(135deg, #172554, #1e3a5f)" },
    ],
  },
  {
    name: "iOS",
    mockup: "iphone",
    description: "iPhone 端随身助理，语音输入、快捷指令、Widget 支持。",
    screenshots: [
      { id: 1, label: "对话页", gradient: "linear-gradient(135deg, #be185d, #ec4899)" },
      { id: 2, label: "快捷操作", gradient: "linear-gradient(135deg, #831843, #9d174d)" },
      { id: 3, label: "Widget", gradient: "linear-gradient(135deg, #500724, #831843)" },
    ],
  },
  {
    name: "微信小程序",
    mockup: "phone",
    description: "微信内直接使用，免安装、快速查询和录入数据。",
    screenshots: [
      { id: 1, label: "首页", gradient: "linear-gradient(135deg, #15803d, #22c55e)" },
      { id: 2, label: "数据录入", gradient: "linear-gradient(135deg, #14532d, #166534)" },
    ],
  },
];

const MOCKUP_CLASS: Record<string, string> = {
  terminal: "mockup-terminal",
  browser: "mockup-browser",
  macbook: "mockup-macbook",
  ipad: "mockup-ipad",
  iphone: "mockup-iphone",
  phone: "mockup-phone",
};

export default function PlatformShowcase() {
  return (
    <section id="platforms">
      <div className="container">
        <p className="section-label">Platforms</p>
        <h2 className="section-title">全平台覆盖</h2>
        <p className="section-desc">
          在任意设备上使用 i-rs-claw，数据通过 i-rs-api 统一同步。
        </p>
        <div className="platforms-list">
          {PLATFORMS.map((p) => (
            <div key={p.name} className="platform-block">
              <div className={`mockup ${MOCKUP_CLASS[p.mockup]}`}>
                <Carousel items={p.screenshots} />
              </div>
              <div className="platform-info">
                <h3>{p.name}</h3>
                <p>{p.description}</p>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
