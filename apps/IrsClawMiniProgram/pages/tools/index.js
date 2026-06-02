const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')

const CATEGORIES = ['i-rs CLI', 'Search', 'Files', 'Memory', 'Vision', 'Visualization', 'Agent', 'MCP', 'Built-in']

const CAT_COLORS = {
  'i-rs CLI': '#FF9500',
  'Search': '#007AFF',
  'Files': '#30B0C0',
  'Memory': '#AF52DE',
  'Vision': '#5856D6',
  'Visualization': '#34C759',
  'Agent': '#FF2D55',
  'MCP': '#00C7BE',
  'Built-in': '#8E8E93'
}

const CAT_ICONS = {
  'i-rs CLI': '⚡',
  'Search': '🔍',
  'Files': '📁',
  'Memory': '🧠',
  'Vision': '👁',
  'Visualization': '📊',
  'Agent': '👤',
  'MCP': '🧩',
  'Built-in': '🔧'
}

function toolCategory(name) {
  if (!name) return 'Built-in'
  if (name.indexOf('i_rs') === 0 || name === 'i-rs') return 'i-rs CLI'
  if (name.indexOf('search') >= 0 || name.indexOf('web') >= 0 || name.indexOf('curl') >= 0) return 'Search'
  if (name.indexOf('file') >= 0 || name.indexOf('semantic') >= 0 || name.indexOf('fs') === 0) return 'Files'
  if (name.indexOf('memory') >= 0 || name.indexOf('skill') >= 0) return 'Memory'
  if (name.indexOf('chart') >= 0) return 'Visualization'
  if (name.indexOf('delegate') >= 0) return 'Agent'
  if (name.indexOf('vision') >= 0) return 'Vision'
  if (name.indexOf('mcp_') === 0) return 'MCP'
  return 'Built-in'
}

Page({
  data: {
    tools: [],
    loaded: false,
    catColors: CAT_COLORS,
    catIcons: CAT_ICONS,
    groups: []
  },

  onLoad: function () {
    helper.bindServerWatcher(this, function () { this.onServerChanged() })
    this.loadTools()
  },

  onShow: function () {
    this.checkServerChanged()
    this.loadTools()
  },

  onServerChanged: function () {
    this.setData({ tools: [], groups: [] })
    this.loadTools()
  },

  goBack: function () {
    wx.navigateBack()
  },

  loadTools: function () {
    const that = this
    api.listTools().then(function (res) {
      const tools = []
      if (res.success && res.data) {
        for (let i = 0; i < res.data.length; i++) {
          const t = res.data[i]
          tools.push({
            id: helper.genId('t'),
            name: (t.function && t.function.name) || t.name || '',
            description: (t.function && t.function.description) || t.description || '',
            category: toolCategory(t.name || (t.function && t.function.name) || '')
          })
        }
      }
      that.setData({ tools: tools, loaded: true })
      that.buildGroups()
    }).catch(function () {
      that.setData({ tools: [], loaded: true, groups: [] })
    })
  },

  hexToRgba: function (hex, alpha) {
    if (!hex || hex[0] !== '#' || hex.length !== 7) return 'rgba(0,0,0,' + (alpha || 0) + ')'
    const r = parseInt(hex.slice(1, 3), 16)
    const g = parseInt(hex.slice(3, 5), 16)
    const b = parseInt(hex.slice(5, 7), 16)
    return 'rgba(' + r + ',' + g + ',' + b + ',' + alpha + ')'
  },

  buildGroups: function () {
    const tools = this.data.tools
    const groups = []
    for (let ci = 0; ci < CATEGORIES.length; ci++) {
      const cat = CATEGORIES[ci]
      const items = []
      for (let i = 0; i < tools.length; i++) {
        if (tools[i].category === cat) items.push(tools[i])
      }
      if (items.length > 0) {
        const color = CAT_COLORS[cat]
        groups.push({
          name: cat,
          items: items,
          count: items.length,
          color: color,
          bgColor: this.hexToRgba(color, 0.12)
        })
      }
    }
    this.setData({ groups: groups })
  }
})
