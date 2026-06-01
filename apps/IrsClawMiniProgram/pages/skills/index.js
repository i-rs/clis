var api = require('../../utils/api.js')
var app = getApp()

Page({
  data: {
    skills: [],
    loaded: false
  },

  onLoad: function() {
    this.lastServerUrl = app.globalData.serverUrl || ''
    this.lastAuthToken = app.globalData.authToken || ''
    this.loadSkills()
  },

  onShow: function() {
    this.checkServerChanged()
    this.loadSkills()
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
    this.setData({ skills: [] })
    this.loadSkills()
  },

  goBack: function() {
    wx.navigateBack()
  },

  loadSkills: function() {
    var that = this
    api.listSkills().then(function(res) {
      if (res.success && res.data) {
        that.setData({ skills: res.data, loaded: true })
      } else {
        that.setData({ skills: [], loaded: true })
      }
    }).catch(function() {
      that.setData({ skills: [], loaded: true })
    })
  }
})
