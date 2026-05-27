var api = require('../../utils/api.js')
var app = getApp()

Page({
  data: {
    isConnected: false,
    theme: 'dark',
    serverUrl: '',
    authToken: '',
    currentAgent: 'default',
    aiConfig: null,
    servers: [],
    currentServerId: '',
    showAddForm: false,
    newName: '',
    newUrl: '',
    newToken: '',
    tokenEditing: false,
    editToken: ''
  },

  onLoad: function() {
    var g = app.globalData
    var servers = wx.getStorageSync('claw_servers') || []
    this.setData({
      theme: g.theme || 'dark',
      serverUrl: g.serverUrl || '',
      authToken: g.authToken || '',
      currentAgent: g.currentAgent || 'default',
      servers: servers,
      currentServerId: wx.getStorageSync('claw_current_server') || ''
    })
    this.checkHealth()
    this.loadConfig()
  },

  goBack: function() { wx.navigateBack() },

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
    var t = this.data.theme === 'dark' ? 'light' : 'dark'
    this.setData({ theme: t })
    app.globalData.theme = t
    app.saveConfig()
  },

  onServerInput: function(e) { this.setData({ serverUrl: e.detail.value }) },
  onTokenInput: function(e) { this.setData({ editToken: e.detail.value }) },

  maskToken: function(token) {
    if (!token) return ''
    if (token.length <= 8) return '••••••••'
    return token.substring(0, 4) + '••••••••' + token.substring(token.length - 4)
  },

  toggleTokenEdit: function() {
    if (this.data.tokenEditing) {
      this.setData({ tokenEditing: false, editToken: '' })
    } else {
      this.setData({ tokenEditing: true, editToken: this.data.authToken })
    }
  },

  confirmToken: function() {
    this.setData({ authToken: this.data.editToken, tokenEditing: false, editToken: '' })
  },

  onSave: function() {
    app.globalData.serverUrl = this.data.serverUrl
    app.globalData.authToken = this.data.authToken
    app.saveConfig()
    this.checkHealth()
    this.loadConfig()
    wx.showToast({ title: '已保存', icon: 'success' })
  },

  toggleAddForm: function() {
    this.setData({ showAddForm: !this.data.showAddForm })
  },

  onNewName: function(e) { this.setData({ newName: e.detail.value }) },
  onNewUrl: function(e) { this.setData({ newUrl: e.detail.value }) },
  onNewToken: function(e) { this.setData({ newToken: e.detail.value }) },

  addServer: function() {
    var name = this.data.newName.trim()
    var url = this.data.newUrl.trim()
    if (!name || !url) {
      wx.showToast({ title: '请填写名称和地址', icon: 'none' })
      return
    }
    var servers = this.data.servers.slice()
    servers.push({
      id: 'srv_' + Date.now(),
      name: name,
      url: url,
      token: this.data.newToken.trim()
    })
    this.setData({
      servers: servers,
      showAddForm: false,
      newName: '',
      newUrl: '',
      newToken: ''
    })
    wx.setStorageSync('claw_servers', servers)
  },

  selectServer: function(e) {
    var id = e.currentTarget.dataset.id
    var servers = this.data.servers
    for (var i = 0; i < servers.length; i++) {
      if (servers[i].id === id) {
        this.setData({
          serverUrl: servers[i].url,
          authToken: servers[i].token || '',
          currentServerId: id
        })
        app.globalData.serverUrl = servers[i].url
        app.globalData.authToken = servers[i].token || ''
        app.saveConfig()
        wx.setStorageSync('claw_current_server', id)
        this.checkHealth()
        this.loadConfig()
        wx.showToast({ title: '已切换', icon: 'success' })
        break
      }
    }
  },

  deleteServer: function(e) {
    var id = e.currentTarget.dataset.id
    var servers = this.data.servers.slice()
    var newServers = []
    for (var i = 0; i < servers.length; i++) {
      if (servers[i].id !== id) newServers.push(servers[i])
    }
    this.setData({ servers: newServers })
    wx.setStorageSync('claw_servers', newServers)
    if (this.data.currentServerId === id) {
      this.setData({ currentServerId: '' })
      wx.removeStorageSync('claw_current_server')
    }
  }
})