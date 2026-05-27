var api = require('../../utils/api.js')
var app = getApp()

Page({
  data: {
    agents: [],
    currentAgent: 'default',
    loaded: false
  },

  onLoad: function() {
    this.setData({ currentAgent: app.globalData.currentAgent || 'default' })
    this.loadAgents()
  },

  goBack: function() {
    wx.navigateBack()
  },

  loadAgents: function() {
    var that = this
    api.listAgents().then(function(res) {
      if (res.success && res.data) {
        that.setData({ agents: res.data, loaded: true })
      } else {
        that.setData({ agents: [], loaded: true })
      }
    }).catch(function() {
      that.setData({ agents: [], loaded: true })
    })
  },

  onSwitchAgent: function(e) {
    var id = e.currentTarget.dataset.id
    app.globalData.currentAgent = id
    app.saveConfig()
    this.setData({ currentAgent: id })
    wx.showToast({ title: '已切换', icon: 'success' })
  }
})
