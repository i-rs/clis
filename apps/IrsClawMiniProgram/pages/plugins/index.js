const app = getApp()

Page({
  data: {
    plugins: []
  },

  onLoad() {
    this.loadPlugins()
  },

  onShow() {
    this.loadPlugins()
  },

  async loadPlugins() {
    const plugins = await app.getPlugins()
    this.setData({ plugins })
  }
})
