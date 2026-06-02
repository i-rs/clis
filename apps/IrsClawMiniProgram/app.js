const STORAGE_KEY = 'claw_config'

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
      const cfg = wx.getStorageSync(STORAGE_KEY)
      if (cfg && typeof cfg === 'object') {
        this.globalData.serverUrl = typeof cfg.serverUrl === 'string' ? cfg.serverUrl : ''
        this.globalData.authToken = typeof cfg.authToken === 'string' ? cfg.authToken : ''
        this.globalData.theme = cfg.theme === 'light' ? 'light' : 'dark'
        this.globalData.currentAgent = typeof cfg.currentAgent === 'string' ? cfg.currentAgent : 'default'
      }
    } catch (e) {
      console.warn('[app] read config failed', e)
    }
    this.applyTheme(this.globalData.theme)
  },

  saveConfig() {
    try {
      wx.setStorageSync(STORAGE_KEY, {
        serverUrl: this.globalData.serverUrl,
        authToken: this.globalData.authToken,
        theme: this.globalData.theme,
        currentAgent: this.globalData.currentAgent
      })
    } catch (e) {
      console.warn('[app] save config failed', e)
    }
  },

  applyTheme(theme) {
    try {
      const pages = getCurrentPages()
      const cls = theme === 'light' ? 'light' : ''
      for (let i = 0; i < pages.length; i++) {
        if (cls) {
          pages[i].selectOwnerComponent && pages[i].selectOwnerComponent()
        }
      }
      if (typeof wx.setNavigationBarColor === 'function') {
        wx.setNavigationBarColor({
          frontColor: theme === 'light' ? '#000000' : '#ffffff',
          backgroundColor: theme === 'light' ? '#F5F6FA' : '#0A0A0F'
        })
      }
    } catch (e) {}
  },

  setTheme(theme) {
    const t = theme === 'light' ? 'light' : 'dark'
    this.globalData.theme = t
    this.saveConfig()
    this.applyTheme(t)
    this.notifyThemeChanged(t)
  },

  notifyServerChanged() {
    try {
      const pages = getCurrentPages()
      for (let i = 0; i < pages.length; i++) {
        const page = pages[i]
        if (page && typeof page.onServerChanged === 'function') {
          page.onServerChanged()
        }
      }
    } catch (e) {}
  },

  notifyThemeChanged(theme) {
    try {
      const pages = getCurrentPages()
      for (let i = 0; i < pages.length; i++) {
        const page = pages[i]
        if (page && typeof page.onThemeChanged === 'function') {
          page.onThemeChanged(theme)
        }
      }
    } catch (e) {}
  }
})
