var api = require('../../utils/api.js')

function formatDate(dateStr) {
  if (!dateStr) return ''
  var d = new Date(dateStr)
  if (isNaN(d.getTime())) return dateStr.substring(0, 10)
  var now = new Date()
  var today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  var yesterday = new Date(today.getTime() - 86400000)
  var weekStart = new Date(today.getTime() - today.getDay() * 86400000)

  if (d >= today) {
    var h = d.getHours(); var m = d.getMinutes()
    return (h < 10 ? '0' : '') + h + ':' + (m < 10 ? '0' : '') + m
  }
  if (d >= yesterday) return 'Yesterday'
  if (d >= weekStart) {
    var days = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat']
    return days[d.getDay()]
  }
  return (d.getMonth() + 1) + '/' + d.getDate()
}

function dateGroup(dateStr) {
  if (!dateStr) return 'Earlier'
  var d = new Date(dateStr)
  if (isNaN(d.getTime())) return 'Earlier'
  var now = new Date()
  var today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  var yesterday = new Date(today.getTime() - 86400000)

  if (d >= today) return 'Today'
  if (d >= yesterday) return 'Yesterday'
  var weekStart = new Date(today.getTime() - today.getDay() * 86400000)
  if (d >= weekStart) return 'This Week'
  return 'Earlier'
}

Page({
  data: {
    groups: [],
    loaded: false
  },

  onLoad: function() {
    this.loadSessions()
  },

  onShow: function() {
    this.loadSessions()
  },

  goBack: function() {
    wx.navigateBack()
  },

  loadSessions: function() {
    var that = this
    api.listSessions().then(function(res) {
      var sessions = []
      if (res.success && res.data) {
        for (var i = 0; i < res.data.length; i++) {
          var s = res.data[i]
          var updated = s.updated_at || s.updatedAt || ''
          sessions.push({
            id: s.id || s.session_id || '',
            title: s.title || 'New Chat',
            messageCount: s.message_count || s.messageCount || 0,
            updatedAt: updated,
            shortDate: formatDate(updated),
            group: dateGroup(updated),
            agentId: s.agent_id || s.agentId || ''
          })
        }
      }
      that.buildGroups(sessions)
      that.setData({ loaded: true })
    }).catch(function() {
      that.setData({ groups: [], loaded: true })
    })
  },

  buildGroups: function(sessions) {
    var order = ['Today', 'Yesterday', 'This Week', 'Earlier']
    var groups = []
    for (var gi = 0; gi < order.length; gi++) {
      var name = order[gi]
      var items = []
      for (var i = 0; i < sessions.length; i++) {
        if (sessions[i].group === name) items.push(sessions[i])
      }
      if (items.length > 0) {
        groups.push({ name: name, items: items })
      }
    }
    this.setData({ groups: groups, sessions: sessions })
  },

  onSwitch: function(e) {
    var id = e.currentTarget.dataset.id
    var app = getApp()
    var that = this
    api.switchSession(id).then(function(res) {
      if (res.success) {
        app.globalData.sessionId = id
        wx.showToast({ title: 'Switched', icon: 'success' })
        setTimeout(function() { wx.navigateBack() }, 400)
      } else {
        wx.showToast({ title: 'Switch failed', icon: 'none' })
      }
    }).catch(function() {
      wx.showToast({ title: 'Network error', icon: 'none' })
    })
  },

  deleteSession: function(e) {
    var id = e.currentTarget.dataset.id
    var that = this
    wx.showModal({
      title: 'Delete session',
      content: 'Are you sure?',
      success: function(res) {
        if (!res.confirm) return
        api.deleteSession(id).then(function(res2) {
          if (res2.success) {
            that.loadSessions()
            wx.showToast({ title: 'Deleted', icon: 'success' })
          }
        }).catch(function() {})
      }
    })
  }
})
