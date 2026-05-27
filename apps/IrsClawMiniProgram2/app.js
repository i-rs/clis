App({
  globalData: {
    serverUrl: '',
    authToken: '',
    theme: 'dark',
    currentAgent: 'default',
    sessionId: null
  },

  onLaunch() {
    try {
      var cfg = wx.getStorageSync('claw_config')
      if (cfg) {
        this.globalData.serverUrl = cfg.serverUrl || ''
        this.globalData.authToken = cfg.authToken || ''
        this.globalData.theme = cfg.theme || 'dark'
        this.globalData.currentAgent = cfg.currentAgent || 'default'
      }
    } catch (e) {}
  },

  saveConfig() {
    try {
      wx.setStorageSync('claw_config', {
        serverUrl: this.globalData.serverUrl,
        authToken: this.globalData.authToken,
        theme: this.globalData.theme,
        currentAgent: this.globalData.currentAgent
      })
    } catch (e) {}
  }
})