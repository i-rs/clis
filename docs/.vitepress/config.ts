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
        text: 'Core Tools',
        collapsed: false,
        items: [
          {
            text: 'i-rs (Unified CLI)',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs/' },
              { text: 'Usage', link: '/crates/i-rs/usage' },
              { text: 'Examples', link: '/crates/i-rs/examples' },
              { text: 'Test', link: '/crates/i-rs/test' }
            ]
          },
          {
            text: 'i-rs-core',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-core/' },
              { text: 'Usage', link: '/crates/i-rs-core/usage' },
              { text: 'Examples', link: '/crates/i-rs-core/examples' },
              { text: 'Test', link: '/crates/i-rs-core/test' }
            ]
          },
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
            text: 'i-rs-todo',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-todo/' },
              { text: 'Usage', link: '/crates/i-rs-todo/usage' },
              { text: 'Examples', link: '/crates/i-rs-todo/examples' },
              { text: 'Test', link: '/crates/i-rs-todo/test' }
            ]
          },
          {
            text: 'i-rs-deploy',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-deploy/' },
              { text: 'Usage', link: '/crates/i-rs-deploy/usage' },
              { text: 'Examples', link: '/crates/i-rs-deploy/examples' },
              { text: 'Test', link: '/crates/i-rs-deploy/test' }
            ]
          }
        ]
      },
      {
        text: 'Health Tracking',
        collapsed: false,
        items: [
          {
            text: 'i-rs-weight',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-weight/' },
              { text: 'Usage', link: '/crates/i-rs-weight/usage' },
              { text: 'Examples', link: '/crates/i-rs-weight/examples' },
              { text: 'Test', link: '/crates/i-rs-weight/test' }
            ]
          },
          {
            text: 'i-rs-mood',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-mood/' },
              { text: 'Usage', link: '/crates/i-rs-mood/usage' },
              { text: 'Examples', link: '/crates/i-rs-mood/examples' },
              { text: 'Test', link: '/crates/i-rs-mood/test' }
            ]
          },
          {
            text: 'i-rs-water',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-water/' },
              { text: 'Usage', link: '/crates/i-rs-water/usage' },
              { text: 'Examples', link: '/crates/i-rs-water/examples' },
              { text: 'Test', link: '/crates/i-rs-water/test' }
            ]
          },
          {
            text: 'i-rs-step',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-step/' },
              { text: 'Usage', link: '/crates/i-rs-step/usage' },
              { text: 'Examples', link: '/crates/i-rs-step/examples' },
              { text: 'Test', link: '/crates/i-rs-step/test' }
            ]
          },
          {
            text: 'i-rs-dose',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-dose/' },
              { text: 'Usage', link: '/crates/i-rs-dose/usage' },
              { text: 'Examples', link: '/crates/i-rs-dose/examples' },
              { text: 'Test', link: '/crates/i-rs-dose/test' }
            ]
          },
          {
            text: 'i-rs-cycle',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-cycle/' },
              { text: 'Usage', link: '/crates/i-rs-cycle/usage' },
              { text: 'Examples', link: '/crates/i-rs-cycle/examples' },
              { text: 'Test', link: '/crates/i-rs-cycle/test' }
            ]
          },
          {
            text: 'i-rs-sit',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-sit/' },
              { text: 'Usage', link: '/crates/i-rs-sit/usage' },
              { text: 'Examples', link: '/crates/i-rs-sit/examples' },
              { text: 'Test', link: '/crates/i-rs-sit/test' }
            ]
          },
          {
            text: 'i-rs-allergy',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-allergy/' },
              { text: 'Usage', link: '/crates/i-rs-allergy/usage' },
              { text: 'Examples', link: '/crates/i-rs-allergy/examples' },
              { text: 'Test', link: '/crates/i-rs-allergy/test' }
            ]
          },
          {
            text: 'i-rs-cal',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-cal/' },
              { text: 'Usage', link: '/crates/i-rs-cal/usage' },
              { text: 'Examples', link: '/crates/i-rs-cal/examples' },
              { text: 'Test', link: '/crates/i-rs-cal/test' }
            ]
          },
          {
            text: 'i-rs-fast',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-fast/' },
              { text: 'Usage', link: '/crates/i-rs-fast/usage' },
              { text: 'Examples', link: '/crates/i-rs-fast/examples' },
              { text: 'Test', link: '/crates/i-rs-fast/test' }
            ]
          },
          {
            text: 'i-rs-habit',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-habit/' },
              { text: 'Usage', link: '/crates/i-rs-habit/usage' },
              { text: 'Examples', link: '/crates/i-rs-habit/examples' },
              { text: 'Test', link: '/crates/i-rs-habit/test' }
            ]
          },
          {
            text: 'i-rs-sleep',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-sleep/' },
              { text: 'Usage', link: '/crates/i-rs-sleep/usage' },
              { text: 'Examples', link: '/crates/i-rs-sleep/examples' },
              { text: 'Test', link: '/crates/i-rs-sleep/test' }
            ]
          },
          {
            text: 'i-rs-exercise',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-exercise/' },
              { text: 'Usage', link: '/crates/i-rs-exercise/usage' },
              { text: 'Examples', link: '/crates/i-rs-exercise/examples' },
              { text: 'Test', link: '/crates/i-rs-exercise/test' }
            ]
          },
          {
            text: 'i-rs-cycling',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-cycling/' },
              { text: 'Usage', link: '/crates/i-rs-cycling/usage' },
              { text: 'Examples', link: '/crates/i-rs-cycling/examples' },
              { text: 'Test', link: '/crates/i-rs-cycling/test' }
            ]
          },
          {
            text: 'i-rs-height',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-height/' },
              { text: 'Usage', link: '/crates/i-rs-height/usage' },
              { text: 'Examples', link: '/crates/i-rs-height/examples' },
              { text: 'Test', link: '/crates/i-rs-height/test' }
            ]
          },
          {
            text: 'i-rs-run',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-run/' },
              { text: 'Usage', link: '/crates/i-rs-run/usage' },
              { text: 'Examples', link: '/crates/i-rs-run/examples' },
              { text: 'Test', link: '/crates/i-rs-run/test' }
            ]
          },
          {
            text: 'i-rs-vision',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-vision/' },
              { text: 'Usage', link: '/crates/i-rs-vision/usage' },
              { text: 'Examples', link: '/crates/i-rs-vision/examples' },
              { text: 'Test', link: '/crates/i-rs-vision/test' }
            ]
          }
        ]
      },
      {
        text: 'Reminders & Expiry',
        collapsed: false,
        items: [
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
            text: 'i-rs-sub',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-sub/' },
              { text: 'Usage', link: '/crates/i-rs-sub/usage' },
              { text: 'Examples', link: '/crates/i-rs-sub/examples' },
              { text: 'Test', link: '/crates/i-rs-sub/test' }
            ]
          },
          {
            text: 'i-rs-bestby',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-bestby/' },
              { text: 'Usage', link: '/crates/i-rs-bestby/usage' },
              { text: 'Examples', link: '/crates/i-rs-bestby/examples' },
              { text: 'Test', link: '/crates/i-rs-bestby/test' }
            ]
          }
        ]
      },
      {
        text: 'Finance & Data',
        collapsed: false,
        items: [
          {
            text: 'i-rs-ledger',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-ledger/' },
              { text: 'Usage', link: '/crates/i-rs-ledger/usage' },
              { text: 'Examples', link: '/crates/i-rs-ledger/examples' },
              { text: 'Test', link: '/crates/i-rs-ledger/test' }
            ]
          },
          {
            text: 'i-rs-recur',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-recur/' },
              { text: 'Usage', link: '/crates/i-rs-recur/usage' },
              { text: 'Examples', link: '/crates/i-rs-recur/examples' },
              { text: 'Test', link: '/crates/i-rs-recur/test' }
            ]
          },
          {
            text: 'i-rs-kv',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-kv/' },
              { text: 'Usage', link: '/crates/i-rs-kv/usage' },
              { text: 'Examples', link: '/crates/i-rs-kv/examples' },
              { text: 'Test', link: '/crates/i-rs-kv/test' }
            ]
          },
          {
            text: 'i-rs-keys',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-keys/' },
              { text: 'Usage', link: '/crates/i-rs-keys/usage' },
              { text: 'Examples', link: '/crates/i-rs-keys/examples' },
              { text: 'Test', link: '/crates/i-rs-keys/test' }
            ]
          },
          {
            text: 'i-rs-invest',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-invest/' },
              { text: 'Usage', link: '/crates/i-rs-invest/usage' },
              { text: 'Examples', link: '/crates/i-rs-invest/examples' },
              { text: 'Test', link: '/crates/i-rs-invest/test' }
            ]
          },
          {
            text: 'i-rs-debt',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-debt/' },
              { text: 'Usage', link: '/crates/i-rs-debt/usage' },
              { text: 'Examples', link: '/crates/i-rs-debt/examples' },
              { text: 'Test', link: '/crates/i-rs-debt/test' }
            ]
          },
          {
            text: 'i-rs-invoice',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-invoice/' },
              { text: 'Usage', link: '/crates/i-rs-invoice/usage' },
              { text: 'Examples', link: '/crates/i-rs-invoice/examples' },
              { text: 'Test', link: '/crates/i-rs-invoice/test' }
            ]
          },
          {
            text: 'i-rs-tax',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-tax/' },
              { text: 'Usage', link: '/crates/i-rs-tax/usage' },
              { text: 'Examples', link: '/crates/i-rs-tax/examples' },
              { text: 'Test', link: '/crates/i-rs-tax/test' }
            ]
          },
          {
            text: 'i-rs-goal',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-goal/' },
              { text: 'Usage', link: '/crates/i-rs-goal/usage' },
              { text: 'Examples', link: '/crates/i-rs-goal/examples' },
              { text: 'Test', link: '/crates/i-rs-goal/test' }
            ]
          }
        ]
      },
      {
        text: 'Food & Lifestyle',
        collapsed: false,
        items: [
          {
            text: 'i-rs-meal',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-meal/' },
              { text: 'Usage', link: '/crates/i-rs-meal/usage' },
              { text: 'Examples', link: '/crates/i-rs-meal/examples' },
              { text: 'Test', link: '/crates/i-rs-meal/test' }
            ]
          },
          {
            text: 'i-rs-pig',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-pig/' },
              { text: 'Usage', link: '/crates/i-rs-pig/usage' },
              { text: 'Examples', link: '/crates/i-rs-pig/examples' },
              { text: 'Test', link: '/crates/i-rs-pig/test' }
            ]
          },
          {
            text: 'i-rs-grocery',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-grocery/' },
              { text: 'Usage', link: '/crates/i-rs-grocery/usage' },
              { text: 'Examples', link: '/crates/i-rs-grocery/examples' },
              { text: 'Test', link: '/crates/i-rs-grocery/test' }
            ]
          }
        ]
      },
      {
        text: 'Personal & Relationships',
        collapsed: false,
        items: [
          {
            text: 'i-rs-gift',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-gift/' },
              { text: 'Usage', link: '/crates/i-rs-gift/usage' },
              { text: 'Examples', link: '/crates/i-rs-gift/examples' },
              { text: 'Test', link: '/crates/i-rs-gift/test' }
            ]
          }
        ]
      },
      {
        text: 'Time & Productivity',
        collapsed: false,
        items: [
          {
            text: 'i-rs-tick',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-tick/' },
              { text: 'Usage', link: '/crates/i-rs-tick/usage' },
              { text: 'Examples', link: '/crates/i-rs-tick/examples' },
              { text: 'Test', link: '/crates/i-rs-tick/test' }
            ]
          },
          {
            text: 'i-rs-spark',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-spark/' },
              { text: 'Usage', link: '/crates/i-rs-spark/usage' },
              { text: 'Examples', link: '/crates/i-rs-spark/examples' },
              { text: 'Test', link: '/crates/i-rs-spark/test' }
            ]
          },
          {
            text: 'i-rs-want',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-want/' },
              { text: 'Usage', link: '/crates/i-rs-want/usage' },
              { text: 'Examples', link: '/crates/i-rs-want/examples' },
              { text: 'Test', link: '/crates/i-rs-want/test' }
            ]
          },
          {
            text: 'i-rs-time',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-time/' },
              { text: 'Usage', link: '/crates/i-rs-time/usage' },
              { text: 'Examples', link: '/crates/i-rs-time/examples' },
              { text: 'Test', link: '/crates/i-rs-time/test' }
            ]
          }
        ]
      },
      {
        text: 'Home & Care',
        collapsed: false,
        items: [
          {
            text: 'i-rs-sheet',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-sheet/' },
              { text: 'Usage', link: '/crates/i-rs-sheet/usage' },
              { text: 'Examples', link: '/crates/i-rs-sheet/examples' },
              { text: 'Test', link: '/crates/i-rs-sheet/test' }
            ]
          },
          {
            text: 'i-rs-toothbrush',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-toothbrush/' },
              { text: 'Usage', link: '/crates/i-rs-toothbrush/usage' },
              { text: 'Examples', link: '/crates/i-rs-toothbrush/examples' },
              { text: 'Test', link: '/crates/i-rs-toothbrush/test' }
            ]
          },
          {
            text: 'i-rs-towel',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-towel/' },
              { text: 'Usage', link: '/crates/i-rs-towel/usage' },
              { text: 'Examples', link: '/crates/i-rs-towel/examples' },
              { text: 'Test', link: '/crates/i-rs-towel/test' }
            ]
          },
          {
            text: 'i-rs-bed',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-bed/' },
              { text: 'Usage', link: '/crates/i-rs-bed/usage' },
              { text: 'Examples', link: '/crates/i-rs-bed/examples' },
              { text: 'Test', link: '/crates/i-rs-bed/test' }
            ]
          },
          {
            text: 'i-rs-ac',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-ac/' },
              { text: 'Usage', link: '/crates/i-rs-ac/usage' },
              { text: 'Examples', link: '/crates/i-rs-ac/examples' },
              { text: 'Test', link: '/crates/i-rs-ac/test' }
            ]
          },
          {
            text: 'i-rs-filter',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-filter/' },
              { text: 'Usage', link: '/crates/i-rs-filter/usage' },
              { text: 'Examples', link: '/crates/i-rs-filter/examples' },
              { text: 'Test', link: '/crates/i-rs-filter/test' }
            ]
          },
          {
            text: 'i-rs-purify',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-purify/' },
              { text: 'Usage', link: '/crates/i-rs-purify/usage' },
              { text: 'Examples', link: '/crates/i-rs-purify/examples' },
              { text: 'Test', link: '/crates/i-rs-purify/test' }
            ]
          },
          {
            text: 'i-rs-plant',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-plant/' },
              { text: 'Usage', link: '/crates/i-rs-plant/usage' },
              { text: 'Examples', link: '/crates/i-rs-plant/examples' },
              { text: 'Test', link: '/crates/i-rs-plant/test' }
            ]
          },
          {
            text: 'i-rs-appliance',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-appliance/' },
              { text: 'Usage', link: '/crates/i-rs-appliance/usage' },
              { text: 'Examples', link: '/crates/i-rs-appliance/examples' },
              { text: 'Test', link: '/crates/i-rs-appliance/test' }
            ]
          }
        ]
      },
      {
        text: 'Pet Care',
        collapsed: false,
        items: [
          {
            text: 'i-rs-feedpet',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-feedpet/' },
              { text: 'Usage', link: '/crates/i-rs-feedpet/usage' },
              { text: 'Examples', link: '/crates/i-rs-feedpet/examples' },
              { text: 'Test', link: '/crates/i-rs-feedpet/test' }
            ]
          },
          {
            text: 'i-rs-petbath',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-petbath/' },
              { text: 'Usage', link: '/crates/i-rs-petbath/usage' },
              { text: 'Examples', link: '/crates/i-rs-petbath/examples' },
              { text: 'Test', link: '/crates/i-rs-petbath/test' }
            ]
          },
          {
            text: 'i-rs-walkdog',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-walkdog/' },
              { text: 'Usage', link: '/crates/i-rs-walkdog/usage' },
              { text: 'Examples', link: '/crates/i-rs-walkdog/examples' },
              { text: 'Test', link: '/crates/i-rs-walkdog/test' }
            ]
          },
          {
            text: 'i-rs-aqua',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-aqua/' },
              { text: 'Usage', link: '/crates/i-rs-aqua/usage' },
              { text: 'Examples', link: '/crates/i-rs-aqua/examples' },
              { text: 'Test', link: '/crates/i-rs-aqua/test' }
            ]
          }
        ]
      },
      {
        text: 'Productivity & Tracking',
        collapsed: false,
        items: [
          {
            text: 'i-rs-read',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-read/' },
              { text: 'Usage', link: '/crates/i-rs-read/usage' },
              { text: 'Examples', link: '/crates/i-rs-read/examples' },
              { text: 'Test', link: '/crates/i-rs-read/test' }
            ]
          },
          {
            text: 'i-rs-movie',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-movie/' },
              { text: 'Usage', link: '/crates/i-rs-movie/usage' },
              { text: 'Examples', link: '/crates/i-rs-movie/examples' },
              { text: 'Test', link: '/crates/i-rs-movie/test' }
            ]
          },
          {
            text: 'i-rs-budget',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-budget/' },
              { text: 'Usage', link: '/crates/i-rs-budget/usage' },
              { text: 'Examples', link: '/crates/i-rs-budget/examples' },
              { text: 'Test', link: '/crates/i-rs-budget/test' }
            ]
          },
          {
            text: 'i-rs-project',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-project/' },
              { text: 'Usage', link: '/crates/i-rs-project/usage' },
              { text: 'Examples', link: '/crates/i-rs-project/examples' },
              { text: 'Test', link: '/crates/i-rs-project/test' }
            ]
          },
          {
            text: 'i-rs-birthday',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-birthday/' },
              { text: 'Usage', link: '/crates/i-rs-birthday/usage' },
              { text: 'Examples', link: '/crates/i-rs-birthday/examples' },
              { text: 'Test', link: '/crates/i-rs-birthday/test' }
            ]
          },
          {
            text: 'i-rs-contact',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-contact/' },
              { text: 'Usage', link: '/crates/i-rs-contact/usage' },
              { text: 'Examples', link: '/crates/i-rs-contact/examples' },
              { text: 'Test', link: '/crates/i-rs-contact/test' }
            ]
          },
          {
            text: 'i-rs-podcast',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-podcast/' },
              { text: 'Usage', link: '/crates/i-rs-podcast/usage' },
              { text: 'Examples', link: '/crates/i-rs-podcast/examples' },
              { text: 'Test', link: '/crates/i-rs-podcast/test' }
            ]
          },
          {
            text: 'i-rs-snippet',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-snippet/' },
              { text: 'Usage', link: '/crates/i-rs-snippet/usage' },
              { text: 'Examples', link: '/crates/i-rs-snippet/examples' },
              { text: 'Test', link: '/crates/i-rs-snippet/test' }
            ]
          },
          {
            text: 'i-rs-article',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-article/' },
              { text: 'Usage', link: '/crates/i-rs-article/usage' },
              { text: 'Examples', link: '/crates/i-rs-article/examples' },
              { text: 'Test', link: '/crates/i-rs-article/test' }
            ]
          },
          {
            text: 'i-rs-car',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-car/' },
              { text: 'Usage', link: '/crates/i-rs-car/usage' },
              { text: 'Examples', link: '/crates/i-rs-car/examples' },
              { text: 'Test', link: '/crates/i-rs-car/test' }
            ]
          },
          {
            text: 'i-rs-quote',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-quote/' },
              { text: 'Usage', link: '/crates/i-rs-quote/usage' },
              { text: 'Examples', link: '/crates/i-rs-quote/examples' },
              { text: 'Test', link: '/crates/i-rs-quote/test' }
            ]
          },
          {
            text: 'i-rs-vocab',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-vocab/' },
              { text: 'Usage', link: '/crates/i-rs-vocab/usage' },
              { text: 'Examples', link: '/crates/i-rs-vocab/examples' },
              { text: 'Test', link: '/crates/i-rs-vocab/test' }
            ]
          },
          {
            text: 'i-rs-event',
            collapsed: true,
            items: [
              { text: 'Overview', link: '/crates/i-rs-event/' },
              { text: 'Usage', link: '/crates/i-rs-event/usage' },
              { text: 'Examples', link: '/crates/i-rs-event/examples' },
              { text: 'Test', link: '/crates/i-rs-event/test' }
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