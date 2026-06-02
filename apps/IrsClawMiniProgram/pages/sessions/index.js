const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')

Page({
  data: {
    groups: [],
    loaded: false
  },

  onLoad: function () {
    helper.bindServerWatcher(this, function () { this.onServerChanged() })
    this.loadSessions()
  },

  onShow: function () {
    this.checkServerChanged()
    this.loadSessions()
  },

  onServerChanged: function () {
    this.setData({ groups: [], sessions: [] })
    this.loadSessions()
  },

  goBack: function () {
    wx.navigateBack()
  },

  goToUsage: function () {
    wx.navigateTo({ url: '/pages/usage/index' })
  },

  loadSessions: function () {
    const that = this
    api.listSessions().then(function (res) {
      const sessions = []
      if (res.success && res.data) {
        for (let i = 0; i < res.data.length; i++) {
          const s = res.data[i]
          const updated = s.updated_at || s.updatedAt || ''
          sessions.push({
            id: s.id || s.session_id || '',
            title: s.title || '新对话',
            messageCount: s.message_count || s.messageCount || 0,
            updatedAt: updated,
            shortDate: helper.formatDate(updated),
            group: helper.dateGroup(updated),
            agentId: s.agent_id || s.agentId || ''
          })
        }
      }
      that.setData({ groups: helper.groupByDate(sessions, 'updatedAt'), sessions: sessions, loaded: true })
    }).catch(function () {
      that.setData({ groups: [], loaded: true })
    })
  },

  onSwitch: function (e) {
    const id = e.currentTarget.dataset.id
    const app = getApp()
    api.switchSession(id).then(function (res) {
      if (res.success) {
        app.globalData.sessionId = id
        wx.showToast({ title: '已切换', icon: 'success' })
        setTimeout(function () { wx.navigateBack() }, 400)
      } else {
        wx.showToast({ title: '切换失败', icon: 'none' })
      }
    }).catch(function () {
      wx.showToast({ title: '网络错误', icon: 'none' })
    })
  },

  deleteSession: function (e) {
    const id = e.currentTarget.dataset.id
    const that = this
    wx.showModal({
      title: '删除会话',
      content: '确认删除该会话?',
      success: function (res) {
        if (!res.confirm) return
        api.deleteSession(id).then(function (res2) {
          if (res2.success) {
            that.loadSessions()
            wx.showToast({ title: '已删除', icon: 'success' })
          }
        }).catch(function () {})
      }
    })
  }
})
