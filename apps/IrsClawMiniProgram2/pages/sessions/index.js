var api = require('../../utils/api.js')

Page({
  data: {
    sessions: [],
    loaded: false
  },

  onLoad: function() {
    this.loadSessions()
  },

  goBack: function() {
    wx.navigateBack()
  },

  loadSessions: function() {
    var that = this
    api.listSessions().then(function(res) {
      if (res.success && res.data) {
        that.setData({ sessions: res.data, loaded: true })
      } else {
        that.setData({ sessions: [], loaded: true })
      }
    }).catch(function() {
      that.setData({ sessions: [], loaded: true })
    })
  },

  onSwitch: function(e) {
    var id = e.currentTarget.dataset.id
    var title = e.currentTarget.dataset.title
    var app = getApp()
    var that = this
    api.switchSession(id).then(function(res) {
      if (res.success) {
        app.globalData.sessionId = id
        wx.showToast({ title: '已切换', icon: 'success' })
        setTimeout(function() {
          wx.navigateBack()
        }, 500)
      } else {
        wx.showToast({ title: '切换失败', icon: 'none' })
      }
    }).catch(function() {
      wx.showToast({ title: '网络错误', icon: 'none' })
    })
  }
})
