App({
  globalData: {
    serverUrl: 'http://localhost:3000',
    authToken: '',
    isConnected: false,
    currentAgentId: 'default',
    currentSession: null,
    sessions: [],
    agents: [],
    tools: [],
    skills: [],
    plugins: [],
    config: null
  },

  onLaunch() {
    const savedUrl = wx.getStorageSync('serverUrl')
    if (savedUrl) this.globalData.serverUrl = savedUrl
    const savedToken = wx.getStorageSync('authToken')
    if (savedToken) this.globalData.authToken = savedToken
  },

  // Unified request wrapper
  request(url, method = 'GET', data = null) {
    const that = this
    return new Promise((resolve, reject) => {
      wx.request({
        url,
        method,
        data,
        header: {
          'Content-Type': 'application/json',
          'Authorization': that.globalData.authToken ? `Bearer ${that.globalData.authToken}` : ''
        },
        success: (res) => {
          if (res.statusCode >= 200 && res.statusCode < 300) {
            // API returns {success, data, error} wrapper
            if (res.data && res.data.success === false) {
              reject(new Error(res.data.error || 'Request failed'))
            } else {
              resolve(res.data)
            }
          } else {
            reject(new Error(`HTTP ${res.statusCode}`))
          }
        },
        fail: reject
      })
    })
  },

  // API methods
  async ping() {
    try {
      await this.request(`${this.globalData.serverUrl}/api/health`)
      this.globalData.isConnected = true
      return true
    } catch (e) {
      this.globalData.isConnected = false
      return false
    }
  },

  async getConfig() {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/config`)
      this.globalData.config = res.data
      return res.data
    } catch (e) {
      return null
    }
  },

  async updateConfig(config) {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/config`, 'PATCH', config)
      this.globalData.config = res.data
      return res.data
    } catch (e) {
      return null
    }
  },

  async getSessions() {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/sessions`)
      this.globalData.sessions = res.data || []
      return this.globalData.sessions
    } catch (e) {
      return []
    }
  },

  async getSession(id) {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/sessions/${id}`)
      return res.data
    } catch (e) {
      return null
    }
  },

  async createSession(agentId) {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/sessions`, 'POST', { agent_id: agentId })
      const session = res.data
      this.globalData.sessions.unshift(session)
      return session
    } catch (e) {
      return null
    }
  },

  async deleteSession(id) {
    try {
      await this.request(`${this.globalData.serverUrl}/api/sessions/${id}`, 'DELETE')
      this.globalData.sessions = this.globalData.sessions.filter(s => s.id !== id)
      return true
    } catch (e) {
      return false
    }
  },

  async sendMessage(content, agentId) {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/chat`, 'POST', {
        message: content,
        agent_id: agentId || this.globalData.currentAgentId
      })
      return res.data
    } catch (e) {
      return null
    }
  },

  async getAgents() {
    // Agent list is not in dashboard API, return default
    return [{ id: 'default', name: 'Default Agent' }]
  },

  async getTools() {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/tools`)
      this.globalData.tools = res.data || []
      return this.globalData.tools
    } catch (e) {
      return []
    }
  },

  async getSkills() {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/skills`)
      this.globalData.skills = res.data || []
      return this.globalData.skills
    } catch (e) {
      return []
    }
  },

  async getPlugins() {
    try {
      const res = await this.request(`${this.globalData.serverUrl}/api/plugins`)
      this.globalData.plugins = res.data || []
      return this.globalData.plugins
    } catch (e) {
      return []
    }
  }
})
