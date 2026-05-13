import { defineConfig } from 'vitepress'

export default defineConfig({
  title: 'i-rs CLI Tools',
  description: 'Cross-platform CLI tools built with Rust',
  appearance: 'dark',
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Tools', link: '/crates/i-rs-server/' }
    ],
    sidebar: [
      {
        text: 'Guide',
        items: [
          { text: 'Getting Started', link: '/guide/getting-started' }
        ]
      },
      {
        text: 'CLI Tools',
        items: [
          { text: 'i-rs-server', link: '/crates/i-rs-server/' },
          { text: 'i-rs-password', link: '/crates/i-rs-password/' },
          { text: 'i-rs-bookmark', link: '/crates/i-rs-bookmark/' },
          { text: 'i-rs-note', link: '/crates/i-rs-note/' },
          { text: 'i-rs-domain', link: '/crates/i-rs-domain/' },
          { text: 'i-rs-remind', link: '/crates/i-rs-remind/' },
          { text: 'i-rs-weight', link: '/crates/i-rs-weight/' }
        ]
      }
    ],
    socialLinks: [
      { icon: 'github', link: 'https://github.com/i-rs/clis' }
    ],
    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2024 i-rs'
    }
  }
})