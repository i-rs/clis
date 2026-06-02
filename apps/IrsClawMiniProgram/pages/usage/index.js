const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')

Page({
  data: {
    stats: null,
    isLoading: true,
    selectedPeriod: 'all',
    periods: [
      { key: 'today', label: '今天' },
      { key: '7d', label: '7天' },
      { key: '30d', label: '30天' },
      { key: 'all', label: '全部' }
    ],
    error: null
  },

  onLoad: function () {
    helper.bindServerWatcher(this, function () { this.onServerChanged() })
    this.loadStats()
  },

  onShow: function () {
    this.checkServerChanged()
    this.loadStats()
  },

  onServerChanged: function () {
    this.setData({ stats: null })
    this.loadStats()
  },

  onPeriodChange: function (e) {
    const period = e.currentTarget.dataset.period
    this.setData({ selectedPeriod: period })
    this.loadStats(period)
  },

  loadStats: function (period) {
    const that = this
    const p = period || this.data.selectedPeriod
    that.setData({ isLoading: true, error: null })
    api.getStats(p).then(function (res) {
      if (res.success && res.data) {
        that.setData({ stats: res.data, isLoading: false })
      } else {
        that.setData({ isLoading: false, error: (res && res.error) || '加载失败' })
      }
    }).catch(function () {
      that.setData({ isLoading: false, error: '网络错误' })
    })
  },

  formatNumber: helper.formatNumber,
  formatCost: helper.formatCost,

  goBack: function () { wx.navigateBack() }
})
