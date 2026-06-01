var api = require('../../utils/api.js')
var app = getApp()

function genId() {
  return 'msg_' + Date.now() + '_' + Math.floor(Math.random() * 10000)
}

function toSingleLine(text) {
  if (!text) return ''
  return String(text).replace(/\s+/g, ' ').trim()
}

function smartTruncate(text, maxLen) {
  if (!text || text.length <= maxLen) return text || ''
  return text.substring(0, maxLen) + '…'
}

function formatJson(text) {
  if (!text) return ''
  var str = typeof text === 'string' ? text : String(text)
  try {
    var obj = JSON.parse(str)
    if (typeof obj === 'object' && obj !== null) {
      return JSON.stringify(obj, null, 2)
    }
  } catch (e) {
    // Not valid JSON, try simple formatting
  }
  // If contains JSON-like structures but invalid, add line breaks
  if (str.length > 80 && (str.indexOf('"') !== -1 || str.indexOf(',') !== -1)) {
    return str
      .replace(/([{,])/g, '$1\n  ')
      .replace(/(":\s*)/g, '$1')
      .replace(/^\s+/gm, '  ')
  }
  return str
}

function charCount(text) {
  if (!text || text.length === 0) return ''
  var count = text.length
  if (count < 1000) return count + 'c'
  return Math.floor(count / 1000) + 'k'
}

Page({
  data: {
    messages: [],
    inputText: '',
    loading: false,
    menuOpen: false,
    isConnected: false,
    noServer: false,
    currentAgent: 'default',
    inputFocused: false,
    sessionId: '',
    sessionTitle: '',
    agents: [],
    scrollTarget: '',
    streamingContent: '',
    streamingReasoning: '',
    streamingToolCalls: [],
    renderTick: 0,
    streamTask: null
  },

  onLoad: function() {
    this.lastServerUrl = app.globalData.serverUrl || ''
    this.lastAuthToken = app.globalData.authToken || ''
    this.checkConnection()
    this.loadAgents()
    this.loadOrCreateSession()
  },

  onShow: function() {
    this.checkServerChanged()
    this.checkConnection()
    this.loadAgents()
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
    if (this.data.streamTask) {
      try { this.data.streamTask.abort() } catch (e) {}
    }
    this.setData({
      messages: [],
      sessionId: '',
      sessionTitle: '',
      streamingContent: '',
      streamingReasoning: '',
      streamingToolCalls: [],
      streamTask: null
    })
    app.globalData.sessionId = null
    this.loadOrCreateSession()
    this.loadAgents()
  },

  onUnload: function() {
    if (this.data.streamTask) {
      this.data.streamTask.abort()
    }
  },

  checkConnection: function() {
    var that = this
    var serverUrl = app.globalData.serverUrl
    if (!serverUrl) {
      that.setData({ isConnected: false, noServer: true })
      return
    }
    that.setData({ noServer: false })
    api.healthCheck().then(function(res) {
      that.setData({ isConnected: !!(res && res.success) })
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
        var tArgs = m.args || ''
        var tResult = m.result || ''
        var tError = m.error || ''
        if (m.success === false && !tError) {
          tError = tResult || '执行失败'
        }
        if (tError) {
          var lowerTErr = tError.toLowerCase()
          if (lowerTErr.indexOf('"success":true') !== -1) {
            tError = ''
          }
        }
        if (!tError && tResult) {
          var lowerTResult = tResult.toLowerCase()
          if (lowerTResult.indexOf('error') !== -1 ||
              lowerTResult.indexOf('failed') !== -1 ||
              lowerTResult.indexOf('failure') !== -1 ||
              lowerTResult.indexOf('panic') !== -1 ||
              lowerTResult.indexOf('执行错误') !== -1) {
            tError = tResult
            tResult = ''
          }
        }
        var tStatus = m.status || (tError ? 'error' : 'done')
        var tDisplayResult = tStatus === 'error' ? '' : formatJson(tResult)
        var tDisplayError = tError
        var tPreviewSource = tStatus === 'error' ? tDisplayError : tResult
        var tPreview = tPreviewSource ? smartTruncate(toSingleLine(tPreviewSource), 40) : ''
        msgs.push({
          id: genId(),
          role: 'tool_call',
          name: m.name || '未知工具',
          args: formatJson(tArgs),
          result: tDisplayResult,
          error: tDisplayError,
          status: tStatus,
          preview: tPreview,
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
            reasoningCount: charCount(reasoning),
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
        var resultStr = toolInfo.result || ''
        var argsStr = toolInfo.arguments || toolInfo.args || ''
        var errorStr = toolInfo.error || ''
        var successFlag = toolInfo.success
        if (successFlag === false && !errorStr) {
          errorStr = resultStr || '执行失败'
        }
        if (errorStr) {
          var lowerErr = errorStr.toLowerCase()
          if (lowerErr.indexOf('"success":true') !== -1) {
            errorStr = ''
          }
        }
        if (!errorStr && resultStr) {
          var lowerResult = resultStr.toLowerCase()
          if (lowerResult.indexOf('error') !== -1 ||
              lowerResult.indexOf('failed') !== -1 ||
              lowerResult.indexOf('failure') !== -1 ||
              lowerResult.indexOf('panic') !== -1 ||
              lowerResult.indexOf('执行错误') !== -1) {
            errorStr = resultStr
            resultStr = ''
          }
        }
        var status = errorStr ? 'error' : 'done'
        var displayResult = status === 'error' ? '' : formatJson(resultStr)
        var displayError = errorStr
        var previewSource = status === 'error' ? displayError : resultStr
        var preview = previewSource ? smartTruncate(toSingleLine(previewSource), 40) : ''
        var toolMsg = {
          id: genId(),
          role: 'tool_call',
          name: toolInfo.name || '未知工具',
          args: formatJson(argsStr),
          result: displayResult,
          error: displayError,
          status: status,
          preview: preview,
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
      reasoningCount: charCount(reasoning),
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
