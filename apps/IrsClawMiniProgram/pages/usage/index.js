var api = require('../../utils/api.js')
var app = getApp()

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

  onLoad: function() {
    this.loadStats()
  },

  onShow: function() {
    this.loadStats()
  },

  onPeriodChange: function(e) {
    var period = e.currentTarget.dataset.period
    this.setData({ selectedPeriod: period })
    this.loadStats(period)
  },

  loadStats: function(period) {
    var that = this
    period = period || this.data.selectedPeriod
    that.setData({ isLoading: true, error: null })
    api.getStats(period).then(function(res) {
      if (res.success && res.data) {
        that.setData({
          stats: res.data,
          isLoading: false
        })
      } else {
        that.setData({ isLoading: false, error: res.error || '加载失败' })
      }
    }).catch(function(err) {
      that.setData({ isLoading: false, error: '网络错误' })
    })
  },

  formatNumber: function(n) {
    if (!n && n !== 0) return '0'
    return n.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
  },

  formatCost: function(cost) {
    if (!cost && cost !== 0) return '$0.00'
    return '$' + cost.toFixed(4)
  },

  goBack: function() { wx.navigateBack() }
})