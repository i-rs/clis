const app = getApp()

Page({
  data: {
    isConnected: false,
    currentAgentId: 'default',
    messages: [],
    inputText: '',
    menuOpen: false
  },

  onLoad() {
    this.checkAndLoad()
  },

  onShow() {
    if (app.globalData.currentSession) {
      this.loadSessionMessages(app.globalData.currentSession.id)
    }
  },

  async checkAndLoad() {
    const connected = await app.ping()
    this.setData({ isConnected: connected })
    app.globalData.isConnected = connected

    if (connected) {
      await app.getConfig()
      const sessions = await app.getSessions()
      if (sessions.length > 0) {
        const latest = sessions[0]
        app.globalData.currentSession = latest
        await this.loadSessionMessages(latest.id)
      }
    }
  },

  async loadSessionMessages(sessionId) {
    const data = await app.getSession(sessionId)
    if (data && data.messages) {
      const msgs = data.messages.map((m, idx) => {
        const id = m.id || `msg-${idx}`
        let toolCalls = m.tool_calls
        if (toolCalls && typeof toolCalls === 'string') {
          try { toolCalls = JSON.parse(toolCalls) } catch (e) { toolCalls = [] }
        }
        if (toolCalls && toolCalls.length > 0) {
          toolCalls = toolCalls.map(tc => {
            if (tc.function && typeof tc.function.arguments === 'string') {
              try { tc.function.arguments = JSON.parse(tc.function.arguments) } catch (e) {}
            }
            return tc
          })
        }
        return {
          ...m,
          id,
          tool_calls: toolCalls,
          _showThinking: false,
          _showTools: false,
          _showReasoning: false,
          _showTool: false
        }
      })
      this.setData({
        messages: msgs,
        currentAgentId: data.meta?.agent_id || 'default'
      })
    }
  },

  onInput(e) {
    this.setData({ inputText: e.detail.value })
  },

  async onSend() {
    const content = this.data.inputText.trim()
    if (!content || !this.data.isConnected) return

    this.setData({ inputText: '' })

    const tempId = `temp-${Date.now()}`
    const msgs = [...this.data.messages, {
      id: tempId,
      role: 'user',
      content,
      _showThinking: false,
      _showTools: false,
      _showReasoning: false
    }]
    this.setData({ messages: msgs })

    const res = await app.sendMessage(content, this.data.currentAgentId)

    if (res && res.session_id) {
      app.globalData.currentSession = { id: res.session_id }
      setTimeout(() => {
        this.loadSessionMessages(res.session_id)
      }, 500)
    }
  },

  onToggleReasoning(e) {
    const id = e.currentTarget.dataset.id
    const msgs = this.data.messages.map(m => {
      if (m.id === id) return { ...m, _showReasoning: !m._showReasoning }
      return m
    })
    this.setData({ messages: msgs })
  },

  onToggleTool(e) {
    const id = e.currentTarget.dataset.id
    const msgs = this.data.messages.map(m => {
      if (m.id === id) return { ...m, _showTool: !m._showTool }
      return m
    })
    this.setData({ messages: msgs })
  },

  onToggleTools(e) {
    const id = e.currentTarget.dataset.id
    const msgs = this.data.messages.map(m => {
      if (m.id === id) return { ...m, _showTools: !m._showTools }
      return m
    })
    this.setData({ messages: msgs })
  },

  onToggleThinking(e) {
    const id = e.currentTarget.dataset.id
    const msgs = this.data.messages.map(m => {
      if (m.id === id) return { ...m, _showThinking: !m._showThinking }
      return m
    })
    this.setData({ messages: msgs })
  },

  async onNewChat() {
    const session = await app.createSession(this.data.currentAgentId)
    if (session) {
      app.globalData.currentSession = session
      this.setData({ messages: [], menuOpen: false })
    }
  },

  onToggleMenu() {
    this.setData({ menuOpen: !this.data.menuOpen })
  },

  onCloseMenu() {
    this.setData({ menuOpen: false })
  },

  onGoAgents() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/agents/index' })
  },

  onGoSessions() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/sessions/index' })
  },

  onGoTools() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/tools/index' })
  },

  onGoSkills() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/skills/index' })
  },

  onGoPlugins() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/plugins/index' })
  },

  onGoSettings() {
    this.setData({ menuOpen: false })
    wx.navigateTo({ url: '/pages/settings/index' })
  }
})
