const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')

Page({
  data: {
    plugins: [],
    loaded: false
  },

  onLoad: function () {
    helper.bindServerWatcher(this, function () { this.onServerChanged() })
    this.loadPlugins()
  },

  onShow: function () {
    this.checkServerChanged()
    this.loadPlugins()
  },

  onServerChanged: function () {
    this.setData({ plugins: [] })
    this.loadPlugins()
  },

  goBack: function () {
    wx.navigateBack()
  },

  loadPlugins: function () {
    const that = this
    api.listPlugins().then(function (res) {
      if (res.success && res.data) {
        that.setData({ plugins: res.data, loaded: true })
      } else {
        that.setData({ plugins: [], loaded: true })
      }
    }).catch(function () {
      that.setData({ plugins: [], loaded: true })
    })
  }
})
