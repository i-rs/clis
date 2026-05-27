var api = require('../../utils/api.js')

Page({
  data: {
    plugins: [],
    loaded: false
  },

  onLoad: function() {
    this.loadPlugins()
  },

  goBack: function() {
    wx.navigateBack()
  },

  loadPlugins: function() {
    var that = this
    api.listPlugins().then(function(res) {
      if (res.success && res.data) {
        that.setData({ plugins: res.data, loaded: true })
      } else {
        that.setData({ plugins: [], loaded: true })
      }
    }).catch(function() {
      that.setData({ plugins: [], loaded: true })
    })
  }
})
