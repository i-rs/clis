const app = getApp()

Page({
  data: {
    isConnected: false,
    serverUrl: '',
    authToken: '',
    maskedToken: '',
    tokenEditing: false,
    newToken: '',
    config: null
  },

  onLoad() {
    this.setData({
      serverUrl: app.globalData.serverUrl,
      authToken: app.globalData.authToken,
      maskedToken: this.maskToken(app.globalData.authToken),
      isConnected: app.globalData.isConnected
    })
    this.loadConfig()
  },

  onShow() {
    this.setData({ isConnected: app.globalData.isConnected })
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

    app.globalData.serverUrl = url
    app.globalData.authToken = finalToken
    wx.setStorageSync('serverUrl', url)
    wx.setStorageSync('authToken', finalToken)

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
  }
})
