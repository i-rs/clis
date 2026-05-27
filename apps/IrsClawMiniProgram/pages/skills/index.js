var api = require('../../utils/api.js')

Page({
  data: {
    skills: [],
    loaded: false
  },

  onLoad: function() {
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
