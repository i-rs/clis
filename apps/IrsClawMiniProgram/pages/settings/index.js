const app = getApp()

Page({
  data: {
    isConnected: false,
    serverUrl: '',
    authToken: '',
    maskedToken: '',
    tokenEditing: false,
    newToken: '',
    config: null,

    showAddForm: false,
    newServerName: '',
    newServerUrl: '',
    newServerToken: '',
    serverList: [],
    currentServerId: null,
    editingServerId: null
  },

  onLoad() {
    this.syncData()
    this.loadConfig()
  },

  onShow() {
    this.setData({ isConnected: app.globalData.isConnected })
  },

  syncData() {
    const list = app.globalData.serverList || []
    this.setData({
      serverUrl: app.globalData.serverUrl,
      authToken: app.globalData.authToken,
      maskedToken: this.maskToken(app.globalData.authToken),
      isConnected: app.globalData.isConnected,
      serverList: list,
      currentServerId: app.globalData.currentServerId
    })
  },

  maskToken(token) {
    if (!token || token.length < 8) return token || ''
    const prefix = token.slice(0, 4)
    const suffix = token.slice(-4)
    return `${prefix}****${suffix}`
  },

  async loadConfig() {
    const config = await app.getConfig()
    if (config) {
      this.setData({ config })
    }
  },

  onUrl(e) {
    this.setData({ serverUrl: e.detail.value })
  },

  onEditToken() {
    this.setData({
      tokenEditing: !this.data.tokenEditing,
      newToken: ''
    })
  },

  onNewToken(e) {
    this.setData({ newToken: e.detail.value })
  },

  onSave() {
    const url = this.data.serverUrl.trim()
    if (!url) {
      wx.showToast({ title: '请输入服务器地址', icon: 'none' })
      return
    }

    const finalToken = this.data.tokenEditing ? this.data.newToken.trim() : this.data.authToken

    const currentId = this.data.currentServerId || app.globalData.currentServerId
    if (currentId) {
      app.updateServer(currentId, { url, token: finalToken })
    } else {
      const name = (url.replace(/^https?:\/\//, '').split('/')[0]).split(':')[0] || 'Server'
      const srv = app.addServer(name, url, finalToken)
      if (srv) app.switchServer(srv.id)
    }

    if (currentId) app.switchServer(currentId)

    this.setData({
      authToken: finalToken,
      maskedToken: this.maskToken(finalToken),
      tokenEditing: false,
      newToken: ''
    })

    app.ping().then(ok => {
      this.setData({ isConnected: ok })
      if (ok) {
        wx.showToast({ title: '连接成功', icon: 'success' })
        this.loadConfig()
      } else {
        wx.showToast({ title: '连接失败', icon: 'none' })
      }
    })
  },

  // ---- Server list actions ----

  onToggleAddForm() {
    this.setData({
      showAddForm: !this.data.showAddForm,
      newServerName: '',
      newServerUrl: '',
      newServerToken: ''
    })
  },

  onNewServerName(e) {
    this.setData({ newServerName: e.detail.value })
  },

  onNewServerUrl(e) {
    this.setData({ newServerUrl: e.detail.value })
  },

  onNewServerToken(e) {
    this.setData({ newServerToken: e.detail.value })
  },

  onAddServer() {
    const name = this.data.newServerName.trim()
    const url = this.data.newServerUrl.trim()
    const token = this.data.newServerToken.trim()
    if (!name || !url) {
      wx.showToast({ title: '名称和地址不能为空', icon: 'none' })
      return
    }
    app.addServer(name, url, token)
    this.setData({
      showAddForm: false,
      serverList: app.globalData.serverList
    })
    wx.showToast({ title: '已添加', icon: 'success' })
  },

  onSelectServer(e) {
    const id = e.currentTarget.dataset.id
    app.switchServer(id)
    this.syncData()
    app.ping().then(ok => {
      this.setData({ isConnected: ok })
      if (ok) {
        wx.showToast({ title: '已切换', icon: 'success' })
        this.loadConfig()
      } else {
        wx.showToast({ title: '连接失败', icon: 'none' })
      }
    })
  },

  onDeleteServer(e) {
    const id = e.currentTarget.dataset.id
    const server = this.data.serverList.find(s => s.id === id)
    wx.showModal({
      title: '删除服务器',
      content: `确定删除 "${server?.name || id}" 吗？`,
      success: (res) => {
        if (res.confirm) {
          app.removeServer(id)
          this.syncData()
        }
      }
    })
  },

  onEditServer(e) {
    const id = e.currentTarget.dataset.id
    this.setData({ editingServerId: this.data.editingServerId === id ? null : id })
  },

  onEditServerName(e) {
    const id = e.currentTarget.dataset.id
    const value = e.detail.value
    app.updateServer(id, { name: value })
    this.setData({ serverList: app.globalData.serverList })
  },

  onEditServerUrl(e) {
    const id = e.currentTarget.dataset.id
    const value = e.detail.value
    app.updateServer(id, { url: value })
    this.setData({ serverList: app.globalData.serverList })
  },

  onEditServerToken(e) {
    const id = e.currentTarget.dataset.id
    const value = e.detail.value
    app.updateServer(id, { token: value })
    this.setData({ serverList: app.globalData.serverList })
  }
})
