const app = getApp()

Page({
  data: { sessions: [] },

  onLoad() { this.load() },
  onShow() { this.load() },

  async load() {
    const sessions = await app.getSessions()
    this.setData({ sessions })
  },

  async onSelect(e) {
    const id = e.currentTarget.dataset.id
    const session = await app.getSession(id)
    if (session) {
      app.globalData.currentSession = { id }
      wx.navigateBack()
    }
  },

  onDelete(e) {
    const id = e.currentTarget.dataset.id
    wx.showModal({
      title: 'Delete',
      content: 'Delete this session?',
      success: async (r) => {
        if (r.confirm) {
          await app.deleteSession(id)
          this.load()
        }
      }
    })
  },

  async onNew() {
    const session = await app.createSession(app.globalData.currentAgentId)
    if (session) {
      app.globalData.currentSession = session
      wx.navigateBack()
    }
  }
})
