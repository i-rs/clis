const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')
const app = getApp()

Page({
  data: {
    messages: [],
    inputText: '',
    loading: false,
    menuOpen: false,
    sidebarOffsetX: 0,
    sidebarTouching: false,
    serverUrlDisplay: '',
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

  onLoad: function () {
    helper.bindServerWatcher(this, function () { this.onServerChanged() })
    this.checkConnection()
    this.loadAgents()
    this.loadOrCreateSession()
  },

  onShow: function () {
    this.checkServerChanged()
    this.checkConnection()
    this.loadAgents()
    this.refreshServerUrlDisplay()
    this.syncSessionIfChanged()
  },

  syncSessionIfChanged: function () {
    const targetId = app.globalData.sessionId || ''
    if (targetId && targetId !== this.data.sessionId) {
      this.loadOrCreateSession()
    }
  },

  onServerChanged: function () {
    const kind = this._configChangeKind || 'server'
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
      streamTask: null,
      loading: false,
      inputText: '',
      currentAgent: app.globalData.currentAgent || 'default'
    })
    app.globalData.sessionId = null
    this.checkConnection()
    this.loadAgents()
    this.refreshServerUrlDisplay()
    if (kind === 'server') {
      this.loadOrCreateSession()
    }
  },

  refreshServerUrlDisplay: function () {
    const url = (app.globalData.serverUrl || '').replace(/^https?:\/\//, '').replace(/\/+$/, '')
    this.setData({ serverUrlDisplay: url || '未配置服务器' })
  },

  onUnload: function () {
    if (this.data.streamTask) {
      this.data.streamTask.abort()
    }
  },

  checkConnection: function () {
    const that = this
    const serverUrl = app.globalData.serverUrl
    if (!serverUrl) {
      that.setData({ isConnected: false, noServer: true })
      return
    }
    that.setData({ noServer: false })
    api.healthCheck().then(function (res) {
      that.setData({ isConnected: !!(res && res.success) })
    })
  },

  loadAgents: function () {
    const that = this
    api.listAgents().then(function (res) {
      if (res.success && res.data) {
        that.setData({ agents: res.data })
      }
    }).catch(function () {})
  },

  loadOrCreateSession: function () {
    const that = this
    const savedSessionId = app.globalData.sessionId
    if (savedSessionId && savedSessionId !== this.data.sessionId) {
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
        streamTask: null,
        loading: false
      })
    }
    if (savedSessionId) {
      api.getSession(savedSessionId).then(function (res) {
        if (res.success && res.data) {
          that.setData({
            sessionId: res.data.id,
            sessionTitle: res.data.title || ''
          })
          that.loadMessages(res.data)
          return
        }
        that.createNewSession()
      }).catch(function () {
        that.createNewSession()
      })
      return
    }
    api.getCurrentSession().then(function (res) {
      if (res.success && res.data && res.data.id) {
        that.setData({
          sessionId: res.data.id,
          sessionTitle: res.data.title || ''
        })
        that.loadMessages(res.data)
      } else {
        that.createNewSession()
      }
    }).catch(function () {
      that.createNewSession()
    })
  },

  loadMessages: function (session) {
    if (!session.messages || session.messages.length === 0) {
      this.setData({ messages: [] })
      return
    }
    const raw = session.messages
    const msgs = []
    for (let i = 0; i < raw.length; i++) {
      const m = raw[i]
      if (m.role === 'user') {
        msgs.push({ id: helper.genId('msg'), role: 'user', content: m.content || '' })
      } else if (m.role === 'tool_call') {
        const tResult = m.result || ''
        let tError = m.error || ''
        if (m.success === false && !tError) {
          tError = tResult || '执行失败'
        }
        if (tError && tError.toLowerCase().indexOf('"success":true') !== -1) {
          tError = ''
        }
        let displayResult = tResult
        if (!tError && tResult) {
          const lower = tResult.toLowerCase()
          if (/error|failed|failure|panic|执行错误/.test(lower)) {
            tError = tResult
            displayResult = ''
          }
        }
        const tStatus = m.status || (tError ? 'error' : 'done')
        const showResult = tStatus === 'error' ? '' : helper.formatJson(displayResult)
        const showError = tError
        const previewSrc = tStatus === 'error' ? showError : displayResult
        const preview = previewSrc ? helper.smartTruncate(helper.toSingleLine(previewSrc), 40) : ''
        msgs.push({
          id: helper.genId('msg'),
          role: 'tool_call',
          name: m.name || '未知工具',
          args: helper.formatJson(m.args || ''),
          result: showResult,
          error: showError,
          status: tStatus,
          preview: preview,
          expanded: false
        })
      } else if (m.role === 'assistant') {
        const content = m.content || ''
        const reasoning = m.reasoning || ''
        if (content || reasoning) {
          msgs.push({
            id: helper.genId('msg'),
            role: 'assistant',
            content: content,
            reasoning: reasoning,
            reasoningCount: helper.charCount(reasoning),
            reasoningExpanded: false
          })
        }
      }
    }
    this.setData({ messages: msgs })
    this.scrollToBottom()
  },

  createNewSession: function () {
    const that = this
    const agentId = app.globalData.currentAgent || 'default'
    api.createSession(agentId).then(function (res) {
      if (res.success && res.data) {
        const sid = res.data.session_id || res.data.id || ''
        that.setData({ sessionId: sid, sessionTitle: '', messages: [] })
        app.globalData.sessionId = sid
      }
    }).catch(function () {
      wx.showToast({ title: '创建会话失败', icon: 'none' })
    })
  },

  onInput: function (e) {
    this.setData({ inputText: e.detail.value })
  },

  onInputFocus: function () {
    this.setData({ inputFocused: true })
    this.onCloseMenu()
  },

  onInputBlur: function () {
    this.setData({ inputFocused: false })
  },

  onSend: function () {
    const text = this.data.inputText.trim()
    if (!text || this.data.loading) return
    if (!app.globalData.serverUrl) {
      wx.showToast({ title: '请先配置服务器', icon: 'none' })
      this.onGoSettings()
      return
    }

    const userMsg = { id: helper.genId('msg'), role: 'user', content: text }
    const messages = this.data.messages.concat([userMsg])
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

    const that = this
    const agentId = app.globalData.currentAgent || 'default'
    api.sendMessage(text, agentId).then(function (res) {
      if (res.success && res.data) {
        const sid = res.data.session_id || that.data.sessionId
        if (!that.data.sessionId && sid) {
          that.setData({ sessionId: sid })
          app.globalData.sessionId = sid
        }
        that.startStreaming(sid)
      } else {
        that.setData({ loading: false })
        wx.showToast({ title: (res && res.error) || '发送失败', icon: 'none' })
      }
    }).catch(function () {
      that.setData({ loading: false })
      wx.showToast({ title: '网络错误', icon: 'none' })
    })
  },

  startStreaming: function (sessionId) {
    const that = this
    if (!sessionId) {
      that.setData({ loading: false })
      wx.showToast({ title: '会话无效', icon: 'none' })
      return
    }

    const flushContent = helper.throttle(function (val) {
      that.setData({ streamingContent: val })
      that.scrollToBottom()
    }, 60)
    const flushReasoning = helper.throttle(function (val) {
      that.setData({ streamingReasoning: val })
      that.scrollToBottom()
    }, 100)

    const streamTask = api.streamChat(sessionId, {
      onToken: function (token) {
        flushContent(that.data.streamingContent + token)
      },
      onReasoning: function (text) {
        flushReasoning(that.data.streamingReasoning + text)
      },
      onStatus: function () {},
      onError: function (err) {
        that.commitStreamMessage()
        that.setData({ loading: false, renderTick: 0 })
        wx.showToast({ title: (err && err.error) || '流式错误', icon: 'none' })
      },
      onDone: function (usage, quality) {
        that.commitStreamMessage(usage)
        // If quality data received separately (not from done event), it's already added by onQuality
        if (quality && !that.data._qualityAdded) {
          const qualityMsg = {
            id: helper.genId('qlt'),
            role: 'quality',
            score: (quality.score !== null && quality.score !== undefined) ? quality.score : '',
            complete: !!quality.complete,
            issues: quality.issues || [],
            referencesValid: !!quality.references_valid
          }
          that.setData({
            messages: that.data.messages.concat([qualityMsg]),
            _qualityAdded: true
          })
        }
        that.setData({ loading: false, renderTick: 0, _qualityAdded: false })
        that.scrollToBottom()
      },
      onNewRound: function () {
        that.commitStreamMessage()
      },
      onToolExecuted: function (toolInfo) {
        const resultStr = toolInfo.result || ''
        const argsStr = toolInfo.arguments || toolInfo.args || ''
        let errorStr = toolInfo.error || ''
        if (toolInfo.success === false && !errorStr) {
          errorStr = resultStr || '执行失败'
        }
        if (errorStr && errorStr.toLowerCase().indexOf('"success":true') !== -1) {
          errorStr = ''
        }
        let displayResult = resultStr
        if (!errorStr && resultStr) {
          const lower = resultStr.toLowerCase()
          if (/error|failed|failure|panic|执行错误/.test(lower)) {
            errorStr = resultStr
            displayResult = ''
          }
        }
        const status = errorStr ? 'error' : 'done'
        const showResult = status === 'error' ? '' : helper.formatJson(displayResult)
        const showError = errorStr
        const previewSrc = status === 'error' ? showError : displayResult
        const preview = previewSrc ? helper.smartTruncate(helper.toSingleLine(previewSrc), 40) : ''
        const toolMsg = {
          id: helper.genId('msg'),
          role: 'tool_call',
          name: toolInfo.name || '未知工具',
          args: helper.formatJson(argsStr),
          result: showResult,
          error: showError,
          status: status,
          preview: preview,
          expanded: false
        }
        const messages = that.data.messages.concat([toolMsg])
        that.setData({ messages: messages, renderTick: Date.now() })
        that.scrollToBottom()
      },
      onEvaluation: function (evalInfo) {
        if (!evalInfo) return
        const evalMsg = {
          id: helper.genId('eva'),
          role: 'evaluation',
          tool: evalInfo.tool || '',
          valid: !!evalInfo.valid,
          issues: evalInfo.issues || []
        }
        const messages = that.data.messages.concat([evalMsg])
        that.setData({ messages: messages, renderTick: Date.now() })
        that.scrollToBottom()
      },
      onQuality: function (qualityInfo) {
        if (!qualityInfo) return
        that.setData({ _qualityAdded: true })
        const qualityMsg = {
          id: helper.genId('qlt'),
          role: 'quality',
          score: (qualityInfo.score !== null && qualityInfo.score !== undefined) ? qualityInfo.score : '',
          complete: !!qualityInfo.complete,
          issues: qualityInfo.issues || [],
          referencesValid: !!qualityInfo.references_valid
        }
        const messages = that.data.messages.concat([qualityMsg])
        that.setData({ messages: messages, renderTick: Date.now() })
        that.scrollToBottom()
      }
    })

    this.setData({ streamTask: streamTask })
  },

  commitStreamMessage: function (tokenUsage) {
    const content = this.data.streamingContent
    const reasoning = this.data.streamingReasoning

    if (!content && !reasoning) {
      this.setData({ renderTick: 0 })
      return
    }

    const aiMsg = {
      id: helper.genId('msg'),
      role: 'assistant',
      content: content,
      reasoning: reasoning,
      reasoningCount: helper.charCount(reasoning),
      reasoningExpanded: false,
      tokenUsage: tokenUsage || null
    }

    const messages = this.data.messages.concat([aiMsg])
    this.setData({
      messages: messages,
      streamingContent: '',
      streamingReasoning: '',
      renderTick: 0
    })
    this.scrollToBottom()
  },

  onFeedback: function (e) {
    const id = e.currentTarget.dataset.id
    const positive = e.currentTarget.dataset.positive === 'true'
    if (!this.data.sessionId) {
      wx.showToast({ title: '无活动会话', icon: 'none' })
      return
    }
    const that = this
    api.postFeedback(this.data.sessionId, positive, '').then(function (res) {
      if (res && res.success) {
        wx.showToast({ title: positive ? '已点赞' : '已点踩', icon: 'success' })
        // Update message to show feedback was given
        const messages = that.data.messages
        for (var i = 0; i < messages.length; i++) {
          if (messages[i].id === id) {
            const key = 'messages[' + i + '].feedbackGiven'
            that.setData({ [key]: true })
            break
          }
        }
      } else {
        wx.showToast({ title: '反馈提交失败', icon: 'none' })
      }
    }).catch(function () {
      wx.showToast({ title: '网络错误', icon: 'none' })
    })
  },

  onNewChat: function () {
    if (this.data.streamTask) {
      this.data.streamTask.abort()
    }
    this.onCloseMenu()
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

  onToggleMenu: function () {
    const willOpen = !this.data.menuOpen
    this.setData({
      menuOpen: willOpen,
      sidebarOffsetX: 0
    })
    if (willOpen) {
      this.refreshServerUrlDisplay()
    }
  },

  onCloseMenu: function () {
    if (this.data.menuOpen) {
      try { wx.vibrateShort({ type: 'light' }) } catch (e) {}
    }
    this.setData({
      menuOpen: false,
      sidebarOffsetX: 0,
      sidebarTouching: false
    })
  },

  onMaskMove: function () {},

  onSidebarTouchStart: function (e) {
    if (!this.data.menuOpen) return
    this._touchStartX = (e.touches && e.touches[0] && e.touches[0].clientX) || 0
    this._touchStartY = (e.touches && e.touches[0] && e.touches[0].clientY) || 0
    this._touchBaseOffset = 0
    this.setData({ sidebarTouching: true })
  },

  onSidebarTouchMove: function (e) {
    if (!this.data.menuOpen) return
    if (!this._touchStartX) return
    const t = e.touches && e.touches[0]
    if (!t) return
    const dx = t.clientX - this._touchStartX
    const dy = t.clientY - this._touchStartY
    if (Math.abs(dy) > Math.abs(dx) * 1.4) return
    if (dx > 0) {
      if (this._touchBaseOffset === 0) {
        this._touchBaseOffset = -this._touchBaseOffset
      }
      return
    }
    const sidebarWidth = 300
    const offset = Math.max(-sidebarWidth, dx)
    this.setData({ sidebarOffsetX: offset })
  },

  onSidebarTouchEnd: function (e) {
    if (!this.data.menuOpen) return
    const changed = e.changedTouches && e.changedTouches[0]
    if (!changed) {
      this.setData({ sidebarTouching: false, sidebarOffsetX: 0 })
      return
    }
    const dx = changed.clientX - this._touchStartX
    const sidebarWidth = 300
    if (dx < -sidebarWidth / 3 || (dx < -40 && dx <= this.data.sidebarOffsetX + 5)) {
      this.setData({
        menuOpen: false,
        sidebarOffsetX: 0,
        sidebarTouching: false
      })
      try { wx.vibrateShort({ type: 'light' }) } catch (e) {}
    } else {
      this.setData({ sidebarOffsetX: 0, sidebarTouching: false })
    }
    this._touchStartX = 0
  },

  onGoSessions: function () { wx.navigateTo({ url: '/pages/sessions/index' }) },
  onGoTools: function () { wx.navigateTo({ url: '/pages/tools/index' }) },
  onGoSkills: function () { wx.navigateTo({ url: '/pages/skills/index' }) },
  onGoPlugins: function () { wx.navigateTo({ url: '/pages/plugins/index' }) },
  onGoAgents: function () { wx.navigateTo({ url: '/pages/agents/index' }) },
  onGoUsage: function () { wx.navigateTo({ url: '/pages/usage/index' }) },
  onGoSettings: function () { wx.navigateTo({ url: '/pages/settings/index' }) },

  onToggleReasoning: function (e) {
    const id = e.currentTarget.dataset.id
    const messages = this.data.messages
    for (let i = 0; i < messages.length; i++) {
      if (messages[i].id === id) {
        const key = 'messages[' + i + '].reasoningExpanded'
        this.setData({ [key]: !messages[i].reasoningExpanded })
        break
      }
    }
  },

  onToggleTool: function (e) {
    const id = e.currentTarget.dataset.id
    const messages = this.data.messages
    for (let i = 0; i < messages.length; i++) {
      if (messages[i].id === id) {
        const key = 'messages[' + i + '].expanded'
        this.setData({ [key]: !messages[i].expanded })
        break
      }
    }
  },

  onSuggest: function (e) {
    const text = e.currentTarget.dataset.text
    this.setData({ inputText: text })
    this.onSend()
  },

  scrollToBottom: function () {
    const that = this
    setTimeout(function () {
      that.setData({ scrollTarget: 'msg-bottom' })
    }, 80)
  }
})
