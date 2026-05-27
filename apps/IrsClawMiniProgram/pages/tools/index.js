var api = require('../../utils/api.js')

Page({
  data: {
    tools: [],
    loaded: false
  },

  onLoad: function() {
    this.loadTools()
  },

  goBack: function() {
    wx.navigateBack()
  },

  loadTools: function() {
    var that = this
    api.listTools().then(function(res) {
      if (res.success && res.data) {
        that.setData({ tools: res.data, loaded: true })
      } else {
        that.setData({ tools: [], loaded: true })
      }
    }).catch(function() {
      that.setData({ tools: [], loaded: true })
    })
  }
})
