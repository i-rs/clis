const app = getApp()

Page({
  data: {
    tools: []
  },

  onLoad() {
    this.loadTools()
  },

  onShow() {
    this.loadTools()
  },

  async loadTools() {
    const tools = await app.getTools()
    this.setData({ tools })
  }
})
