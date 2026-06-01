var api = require('../../utils/api.js')
var app = getApp()

Page({
  data: {
    plugins: [],
    loaded: false
  },

  onLoad: function() {
    this.lastServerUrl = app.globalData.serverUrl || ''
    this.lastAuthToken = app.globalData.authToken || ''
    this.loadPlugins()
  },

  onShow: function() {
    this.checkServerChanged()
    this.loadPlugins()
  },

  checkServerChanged: function() {
    var curUrl = app.globalData.serverUrl || ''
    var curToken = app.globalData.authToken || ''
    if (this.lastServerUrl !== curUrl || this.lastAuthToken !== curToken) {
      this.lastServerUrl = curUrl
      this.lastAuthToken = curToken
      this.onServerChanged()
    }
  },

  onServerChanged: function() {
    this.setData({ plugins: [] })
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
