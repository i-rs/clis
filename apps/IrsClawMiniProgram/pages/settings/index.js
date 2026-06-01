var api = require('../../utils/api.js')
var app = getApp()

function maskToken(token) {
  if (!token) return ''
  if (token.length <= 8) return '••••••••'
  return token.substring(0, 4) + '••••••••' + token.substring(token.length - 4)
}

function enrichServers(servers) {
  for (var i = 0; i < servers.length; i++) {
    servers[i].tokenDisplay = maskToken(servers[i].token || '')
  }
  return servers
}

Page({
  data: {
    isConnected: false,
    theme: 'dark',
    displayUrl: '',
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
    showManual: false,
    manualUrl: '',
    manualToken: ''
  },

  onLoad: function() {
    var g = app.globalData
    var servers = enrichServers(wx.getStorageSync('claw_servers') || [])
    var currentId = wx.getStorageSync('claw_current_server') || ''
    var activeUrl = g.serverUrl || ''
    var activeToken = g.authToken || ''

    this.setData({
      theme: g.theme || 'dark',
      serverUrl: activeUrl,
      authToken: activeToken,
      displayUrl: activeUrl || '未设置',
      currentAgent: g.currentAgent || 'default',
      servers: servers,
      currentServerId: currentId
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

  saveServers: function(servers) {
    wx.setStorageSync('claw_servers', servers)
  },

  activateServer: function(id) {
    var servers = this.data.servers
    for (var i = 0; i < servers.length; i++) {
      if (servers[i].id === id) {
        var srv = servers[i]
        app.globalData.serverUrl = srv.url
        app.globalData.authToken = srv.token || ''
        app.saveConfig()
        wx.setStorageSync('claw_current_server', id)
        this.setData({
          currentServerId: id,
          serverUrl: srv.url,
          authToken: srv.token || '',
          displayUrl: srv.url
        })
        this.checkHealth()
        this.loadConfig()
        app.notifyServerChanged()
        return
      }
    }
  },

  selectServer: function(e) {
    var id = e.currentTarget.dataset.id
    this.activateServer(id)
  },

  deleteServer: function(e) {
    var id = e.currentTarget.dataset.id
    var name = e.currentTarget.dataset.name
    var that = this

    wx.showModal({
      title: '删除服务器',
      content: '确认删除「' + name + '」？',
      success: function(res) {
        if (!res.confirm) return
        var servers = that.data.servers.slice()
        var newServers = []
        for (var i = 0; i < servers.length; i++) {
          if (servers[i].id !== id) newServers.push(servers[i])
        }
        enrichServers(newServers)
        that.setData({ servers: newServers })
        that.saveServers(newServers)

        if (that.data.currentServerId === id) {
          wx.removeStorageSync('claw_current_server')
          if (newServers.length > 0) {
            that.activateServer(newServers[0].id)
          } else {
            app.globalData.serverUrl = ''
            app.globalData.authToken = ''
            app.saveConfig()
            that.setData({
              currentServerId: '',
              serverUrl: '',
              authToken: '',
              displayUrl: '未设置'
            })
            that.checkHealth()
          }
        }
        wx.showToast({ title: '已删除', icon: 'success' })
      }
    })
  },

  // Add Server
  toggleAddForm: function() {
    this.setData({
      showAddForm: !this.data.showAddForm,
      showManual: false
    })
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
    var id = 'srv_' + Date.now()
    var srv = {
      id: id,
      name: name,
      url: url,
      token: this.data.newToken.trim()
    }
    srv.tokenDisplay = maskToken(srv.token)
    servers.push(srv)
    this.setData({
      servers: servers,
      showAddForm: false,
      newName: '',
      newUrl: '',
      newToken: ''
    })
    this.saveServers(servers)
    this.activateServer(id)
    wx.showToast({ title: '已添加并连接', icon: 'success' })
  },

  // Manual Connect
  toggleManual: function() {
    this.setData({
      showManual: !this.data.showManual,
      showAddForm: false
    })
  },

  onManualUrl: function(e) { this.setData({ manualUrl: e.detail.value }) },
  onManualToken: function(e) { this.setData({ manualToken: e.detail.value }) },

  manualConnect: function() {
    var url = this.data.manualUrl.trim()
    if (!url) {
      wx.showToast({ title: '请输入服务器地址', icon: 'none' })
      return
    }
    var token = this.data.manualToken.trim()
    app.globalData.serverUrl = url
    app.globalData.authToken = token
    app.saveConfig()

    wx.removeStorageSync('claw_current_server')
    this.setData({
      currentServerId: '',
      serverUrl: url,
      authToken: token,
      displayUrl: url,
      showManual: false,
      manualUrl: '',
      manualToken: ''
    })
    this.checkHealth()
    this.loadConfig()
    app.notifyServerChanged()
    wx.showToast({ title: '已连接', icon: 'success' })
  }
})
