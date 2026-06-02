const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')
const app = getApp()

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

  onLoad: function () {
    const g = app.globalData
    const servers = enrichServers(wx.getStorageSync('claw_servers') || [])
    const currentId = wx.getStorageSync('claw_current_server') || ''
    const activeUrl = g.serverUrl || ''
    const activeToken = g.authToken || ''

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

  onThemeChanged: function (theme) {
    this.setData({ theme: theme })
  },

  goBack: function () { wx.navigateBack() },

  checkHealth: function () {
    const that = this
    const serverUrl = app.globalData.serverUrl
    if (!serverUrl) {
      that.setData({ isConnected: false })
      return
    }
    api.healthCheck().then(function (res) {
      that.setData({ isConnected: !!(res && res.success) })
    })
  },

  loadConfig: function () {
    const that = this
    api.getConfig().then(function (res) {
      if (res.success && res.data) {
        that.setData({ aiConfig: res.data })
      }
    }).catch(function () {})
  },

  toggleTheme: function () {
    app.setTheme(this.data.theme === 'dark' ? 'light' : 'dark')
  },

  saveServers: function (servers) {
    wx.setStorageSync('claw_servers', servers)
  },

  activateServer: function (id) {
    const servers = this.data.servers
    for (let i = 0; i < servers.length; i++) {
      if (servers[i].id === id) {
        const srv = servers[i]
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

  selectServer: function (e) {
    this.activateServer(e.currentTarget.dataset.id)
  },

  deleteServer: function (e) {
    const id = e.currentTarget.dataset.id
    const name = e.currentTarget.dataset.name
    const that = this

    wx.showModal({
      title: '删除服务器',
      content: '确认删除「' + name + '」?',
      success: function (res) {
        if (!res.confirm) return
        const servers = that.data.servers.slice()
        const newServers = []
        for (let i = 0; i < servers.length; i++) {
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

  toggleAddForm: function () {
    this.setData({
      showAddForm: !this.data.showAddForm,
      showManual: false
    })
  },

  onNewName: function (e) { this.setData({ newName: e.detail.value }) },
  onNewUrl: function (e) { this.setData({ newUrl: e.detail.value }) },
  onNewToken: function (e) { this.setData({ newToken: e.detail.value }) },

  addServer: function () {
    const name = this.data.newName.trim()
    const url = this.data.newUrl.trim()
    if (!name || !url) {
      wx.showToast({ title: '请填写名称和地址', icon: 'none' })
      return
    }
    if (!helper.isValidUrl(url)) {
      wx.showToast({ title: '地址格式不正确,需以 http/https 开头', icon: 'none' })
      return
    }
    const servers = this.data.servers.slice()
    const id = helper.genId('srv')
    const srv = {
      id: id,
      name: name,
      url: url,
      token: this.data.newToken.trim()
    }
    srv.tokenDisplay = helper.maskToken(srv.token)
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

  toggleManual: function () {
    this.setData({
      showManual: !this.data.showManual,
      showAddForm: false
    })
  },

  onManualUrl: function (e) { this.setData({ manualUrl: e.detail.value }) },
  onManualToken: function (e) { this.setData({ manualToken: e.detail.value }) },

  manualConnect: function () {
    const url = this.data.manualUrl.trim()
    if (!url) {
      wx.showToast({ title: '请输入服务器地址', icon: 'none' })
      return
    }
    if (!helper.isValidUrl(url)) {
      wx.showToast({ title: '地址格式不正确,需以 http/https 开头', icon: 'none' })
      return
    }
    const token = this.data.manualToken.trim()
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

function enrichServers(servers) {
  if (!Array.isArray(servers)) return []
  for (let i = 0; i < servers.length; i++) {
    servers[i].tokenDisplay = helper.maskToken(servers[i].token || '')
  }
  return servers
}
