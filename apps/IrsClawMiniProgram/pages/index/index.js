var api = require('../../utils/api.js')
var app = getApp()

function genId() {
  return 'msg_' + Date.now() + '_' + Math.floor(Math.random() * 10000)
}

Page({
  data: {
    messages: [],
    inputText: '',
    loading: false,
    menuOpen: false,
    isConnected: false,
    currentAgent: 'default',
    sessionId: '',
    sessionTitle: '',
    agents: [],
    inputFocused: false,
    scrollTarget: '',
    streamingContent: '',
    streamingReasoning: '',
    streamingToolCalls: [],
    renderTick: 0,
    streamTask: null,
    floatBtnMinimized: false
  },

  onLoad: function() {
    this.checkConnection()
    this.loadAgents()
    this.loadOrCreateSession()
  },

  onUnload: function() {
    if (this.data.streamTask) {
      this.data.streamTask.abort()
    }
  },

  checkConnection: function() {
    var that = this
    api.healthCheck().then(function() {
      that.setData({ isConnected: true })
    }).catch(function() {
      that.setData({ isConnected: false })
    })
  },

  loadAgents: function() {
    var that = this
    api.listAgents().then(function(res) {
      if (res.success && res.data) {
        that.setData({ agents: res.data })
      }
    }).catch(function() {})
  },

  loadOrCreateSession: function() {
    var that = this
    var savedSessionId = app.globalData.sessionId
    if (savedSessionId) {
      api.getSession(savedSessionId).then(function(res) {
        if (res.success && res.data) {
          that.setData({
            sessionId: res.data.id,
            sessionTitle: res.data.title || ''
          })
          that.loadMessages(res.data)
          return
        }
        that.createNewSession()
      }).catch(function() {
        that.createNewSession()
      })
      return
    }
    api.getCurrentSession().then(function(res) {
      if (res.success && res.data && res.data.id) {
        that.setData({
          sessionId: res.data.id,
          sessionTitle: res.data.title || ''
        })
        that.loadMessages(res.data)
      } else {
        that.createNewSession()
      }
    }).catch(function() {
      that.createNewSession()
    })
  },

  loadMessages: function(session) {
    if (!session.messages || session.messages.length === 0) {
      this.setData({ messages: [] })
      return
    }
    var raw = session.messages
    var msgs = []
    for (var i = 0; i < raw.length; i++) {
      var m = raw[i]
      if (m.role === 'user') {
        msgs.push({ id: genId(), role: 'user', content: m.content || '' })
      } else if (m.role === 'tool_call') {
        msgs.push({
          id: genId(),
          role: 'tool_call',
          name: m.name || '',
          args: m.args || '',
          result: m.result || '',
          expanded: false
        })
      } else if (m.role === 'assistant') {
        var content = m.content || ''
        var reasoning = m.reasoning || ''
        if (content || reasoning) {
          msgs.push({
            id: genId(),
            role: 'assistant',
            content: content,
            reasoning: reasoning,
            reasoningExpanded: false
          })
        }
      }
    }
    this.setData({ messages: msgs })
    this.scrollToBottom()
  },

  createNewSession: function() {
    var that = this
    var agentId = app.globalData.currentAgent || 'default'
    api.createSession(agentId).then(function(res) {
      if (res.success && res.data) {
        var sid = res.data.session_id || res.data.id || ''
        that.setData({
          sessionId: sid,
          sessionTitle: '',
          messages: []
        })
        app.globalData.sessionId = sid
      }
    }).catch(function() {
      wx.showToast({ title: '创建会话失败', icon: 'none' })
    })
  },

  onInput: function(e) {
    this.setData({ inputText: e.detail.value })
  },

  onInputFocus: function() {
    this.setData({ inputFocused: true })
    this.onCloseMenu()
  },

  onInputBlur: function() {
    this.setData({ inputFocused: false })
  },

  onSend: function() {
    var text = this.data.inputText.trim()
    if (!text || this.data.loading) return

    var userMsg = {
      id: genId(),
      role: 'user',
      content: text
    }

    var messages = this.data.messages.concat([userMsg])
    this.setData({
      messages: messages,
      inputText: '',
      loading: true,
      streamingContent: '',
      streamingReasoning: '',
      streamingToolCalls: [],
      renderTick: 0
    })
    this.scrollToBottom()

    var that = this
    var agentId = app.globalData.currentAgent || 'default'
    api.sendMessage(text, agentId).then(function(res) {
      if (res.success && res.data) {
        var sid = res.data.session_id || that.data.sessionId
        if (!that.data.sessionId && sid) {
          that.setData({ sessionId: sid })
          app.globalData.sessionId = sid
        }
        that.startStreaming(sid)
      } else {
        that.setData({ loading: false })
        wx.showToast({ title: '发送失败', icon: 'none' })
      }
    }).catch(function(err) {
      that.setData({ loading: false })
      wx.showToast({ title: '网络错误', icon: 'none' })
    })
  },

  startStreaming: function(sessionId) {
    var that = this

    var streamTask = api.streamChat(sessionId, {
      onToken: function(token) {
        that.setData({
          streamingContent: that.data.streamingContent + token
        })
        that.scrollToBottom()
      },

      onReasoning: function(text) {
        that.setData({
          streamingReasoning: that.data.streamingReasoning + text
        })
        that.scrollToBottom()
      },

      onStatus: function(status) {},

      onError: function(err) {
        that.commitStreamMessage()
        that.setData({ loading: false, renderTick: 0 })
        wx.showToast({ title: '流式错误', icon: 'none' })
      },

      onDone: function(usage) {
        that.commitStreamMessage()
        that.setData({ loading: false, renderTick: 0 })
      },

      onNewRound: function() {
        that.commitStreamMessage()
      },

      onToolExecuted: function(toolInfo) {
        var toolMsg = {
          id: genId(),
          role: 'tool_call',
          name: toolInfo.name || 'unknown',
          args: toolInfo.arguments || toolInfo.args || '',
          result: toolInfo.result || '',
          expanded: false
        }
        var messages = that.data.messages.concat([toolMsg])
        var tick = Date.now()
        that.setData({
          messages: messages,
          renderTick: tick
        })
        that.scrollToBottom()
      }
    })

    this.setData({ streamTask: streamTask })
  },

  commitStreamMessage: function() {
    var content = this.data.streamingContent
    var reasoning = this.data.streamingReasoning

    if (!content && !reasoning) {
      this.setData({ renderTick: 0 })
      return
    }

    var aiMsg = {
      id: genId(),
      role: 'assistant',
      content: content,
      reasoning: reasoning,
      reasoningExpanded: false
    }

    var messages = this.data.messages.concat([aiMsg])
    this.setData({
      messages: messages,
      streamingContent: '',
      streamingReasoning: '',
      renderTick: 0
    })
    this.scrollToBottom()
  },

  onNewChat: function() {
    if (this.data.streamTask) {
      this.data.streamTask.abort()
    }
    this.setData({
      loading: false,
      streamingContent: '',
      streamingReasoning: '',
      streamingToolCalls: [],
      renderTick: 0,
      streamTask: null
    })
    this.createNewSession()
  },

  onToggleMenu: function() {
    this.setData({ menuOpen: !this.data.menuOpen })
  },

  onCloseMenu: function() {
    this.setData({ menuOpen: false })
  },

  onFloatBtnTap: function() {
    if (this.data.floatBtnMinimized) {
      this.setData({ floatBtnMinimized: false })
      return
    }
    if (this.data.menuOpen) {
      this.setData({ menuOpen: false, floatBtnMinimized: true })
    } else {
      this.setData({ menuOpen: true })
    }
  },

  onScroll: function() {
    if (!this.data.floatBtnMinimized) {
      this.setData({ floatBtnMinimized: true })
    }
  },

  onGoSessions: function() {
    this.onCloseMenu()
    wx.navigateTo({ url: '/pages/sessions/index' })
  },

  onGoTools: function() {
    this.onCloseMenu()
    wx.navigateTo({ url: '/pages/tools/index' })
  },

  onGoSkills: function() {
    this.onCloseMenu()
    wx.navigateTo({ url: '/pages/skills/index' })
  },

  onGoPlugins: function() {
    this.onCloseMenu()
    wx.navigateTo({ url: '/pages/plugins/index' })
  },

  onGoAgents: function() {
    this.onCloseMenu()
    wx.navigateTo({ url: '/pages/agents/index' })
  },

  onGoSettings: function() {
    this.onCloseMenu()
    wx.navigateTo({ url: '/pages/settings/index' })
  },

  onToggleReasoning: function(e) {
    var id = e.currentTarget.dataset.id
    var messages = this.data.messages
    for (var i = 0; i < messages.length; i++) {
      if (messages[i].id === id) {
        var key = 'messages[' + i + '].reasoningExpanded'
        this.setData({ [key]: !messages[i].reasoningExpanded })
        break
      }
    }
  },

  onToggleTool: function(e) {
    var id = e.currentTarget.dataset.id
    var messages = this.data.messages
    for (var i = 0; i < messages.length; i++) {
      if (messages[i].id === id) {
        var key = 'messages[' + i + '].expanded'
        this.setData({ [key]: !messages[i].expanded })
        break
      }
    }
  },

  onSuggest: function(e) {
    var text = e.currentTarget.dataset.text
    this.setData({ inputText: text })
    this.onSend()
  },

  scrollToBottom: function() {
    var that = this
    setTimeout(function() {
      that.setData({ scrollTarget: 'msg-bottom' })
    }, 80)
  }
})
