const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')

Page({
  data: {
    skills: [],
    loaded: false
  },

  onLoad: function () {
    helper.bindServerWatcher(this, function () { this.onServerChanged() })
    this.loadSkills()
  },

  onShow: function () {
    this.checkServerChanged()
    this.loadSkills()
  },

  onServerChanged: function () {
    this.setData({ skills: [] })
    this.loadSkills()
  },

  goBack: function () {
    wx.navigateBack()
  },

  loadSkills: function () {
    const that = this
    api.listSkills().then(function (res) {
      if (res.success && res.data) {
        that.setData({ skills: res.data, loaded: true })
      } else {
        that.setData({ skills: [], loaded: true })
      }
    }).catch(function () {
      that.setData({ skills: [], loaded: true })
    })
  }
})
