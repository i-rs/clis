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
        collapsed: false,
        items: [
          {
            text: 'i-rs-server',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-server/' },
              { text: 'Usage', link: '/crates/i-rs-server/usage' },
              { text: 'Examples', link: '/crates/i-rs-server/examples' },
              { text: 'Test', link: '/crates/i-rs-server/test' }
            ]
          },
          {
            text: 'i-rs-password',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-password/' },
              { text: 'Usage', link: '/crates/i-rs-password/usage' },
              { text: 'Examples', link: '/crates/i-rs-password/examples' },
              { text: 'Test', link: '/crates/i-rs-password/test' }
            ]
          },
          {
            text: 'i-rs-bookmark',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-bookmark/' },
              { text: 'Usage', link: '/crates/i-rs-bookmark/usage' },
              { text: 'Examples', link: '/crates/i-rs-bookmark/examples' },
              { text: 'Test', link: '/crates/i-rs-bookmark/test' }
            ]
          },
          {
            text: 'i-rs-note',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-note/' },
              { text: 'Usage', link: '/crates/i-rs-note/usage' },
              { text: 'Examples', link: '/crates/i-rs-note/examples' },
              { text: 'Test', link: '/crates/i-rs-note/test' }
            ]
          },
          {
            text: 'i-rs-domain',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-domain/' },
              { text: 'Usage', link: '/crates/i-rs-domain/usage' },
              { text: 'Examples', link: '/crates/i-rs-domain/examples' },
              { text: 'Test', link: '/crates/i-rs-domain/test' }
            ]
          },
          {
            text: 'i-rs-remind',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-remind/' },
              { text: 'Usage', link: '/crates/i-rs-remind/usage' },
              { text: 'Examples', link: '/crates/i-rs-remind/examples' },
              { text: 'Test', link: '/crates/i-rs-remind/test' }
            ]
          },
          {
            text: 'i-rs-weight',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-weight/' },
              { text: 'Usage', link: '/crates/i-rs-weight/usage' },
              { text: 'Examples', link: '/crates/i-rs-weight/examples' },
              { text: 'Test', link: '/crates/i-rs-weight/test' }
            ]
          }
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