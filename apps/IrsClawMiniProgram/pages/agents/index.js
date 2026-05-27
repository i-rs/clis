const app = getApp()

Page({
  data: {
    agents: [],
    currentAgentId: null
  },

  onLoad() {
    this.setData({ currentAgentId: app.globalData.currentAgentId })
    this.loadAgents()
  },

  onShow() {
    this.setData({ currentAgentId: app.globalData.currentAgentId })
    this.loadAgents()
  },

  async loadAgents() {
    const agents = await app.getAgents()
    this.setData({ agents })
  },

  onSelectAgent(e) {
    const agentId = e.currentTarget.dataset.id
    app.globalData.currentAgentId = agentId
    this.setData({ currentAgentId: agentId })
    wx.navigateBack()
  }
})
