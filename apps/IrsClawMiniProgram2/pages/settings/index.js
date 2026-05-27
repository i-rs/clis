var api = require('../../utils/api.js')
var app = getApp()

Page({
  data: {
    isConnected: false,
    theme: 'dark',
    serverUrl: '',
    authToken: '',
    currentAgent: 'default',
    aiConfig: null
  },

  onLoad: function() {
    var g = app.globalData
    this.setData({
      theme: g.theme || 'dark',
      serverUrl: g.serverUrl || '',
      authToken: g.authToken || '',
      currentAgent: g.currentAgent || 'default'
    })
    this.checkHealth()
    this.loadConfig()
  },

  goBack: function() {
    wx.navigateBack()
  },

  checkHealth: function() {
    var that = this
    api.healthCheck().then(function() {
      that.setData({ isConnected: true })
    }).catch(function() {
      that.setData({ isConnected: false })
    })
  },

  loadConfig: function() {
    var that = this
    api.getConfig().then(function(res) {
      if (res.success && res.data) {
        that.setData({ aiConfig: res.data })
      }
    }).catch(function() {})
  },

  toggleTheme: function() {
    var newTheme = this.data.theme === 'dark' ? 'light' : 'dark'
    this.setData({ theme: newTheme })
    app.globalData.theme = newTheme
    app.saveConfig()
  },

  onServerInput: function(e) {
    this.setData({ serverUrl: e.detail.value })
  },

  onTokenInput: function(e) {
    this.setData({ authToken: e.detail.value })
  },

  onSave: function() {
    app.globalData.serverUrl = this.data.serverUrl
    app.globalData.authToken = this.data.authToken
    app.saveConfig()
    this.checkHealth()
    wx.showToast({ title: '已保存', icon: 'success' })
  }
})
