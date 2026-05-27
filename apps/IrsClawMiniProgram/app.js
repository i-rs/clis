App({
  globalData: {
    serverUrl: 'http://localhost:3000',
    authToken: '',
    isConnected: false,
    currentServerId: null,
    serverList: [],
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
    const savedList = wx.getStorageSync('serverList')
    if (savedList && savedList.length > 0) {
      this.globalData.serverList = savedList
      const savedId = wx.getStorageSync('currentServerId')
      if (savedId) {
        const server = savedList.find(s => s.id === savedId)
        if (server) {
          this.globalData.currentServerId = savedId
          this.globalData.serverUrl = server.url
          this.globalData.authToken = server.token
        }
      }
    } else {
      const savedUrl = wx.getStorageSync('serverUrl')
      if (savedUrl) this.globalData.serverUrl = savedUrl
      const savedToken = wx.getStorageSync('authToken')
      if (savedToken) this.globalData.authToken = savedToken
      // Migrate legacy single-server to new format
      if (savedUrl) {
        const id = 'srv_' + Date.now()
        const server = { id, name: '默认服务器', url: savedUrl, token: savedToken || '' }
        this.globalData.serverList = [server]
        this.globalData.currentServerId = id
        wx.setStorageSync('serverList', [server])
        wx.setStorageSync('currentServerId', id)
      }
    }
  },

  _saveServerList() {
    wx.setStorageSync('serverList', this.globalData.serverList)
    wx.setStorageSync('currentServerId', this.globalData.currentServerId)
  },

  addServer(name, url, token) {
    const id = 'srv_' + Date.now()
    const server = { id, name, url, token }
    this.globalData.serverList.push(server)
    this._saveServerList()
    return server
  },

  removeServer(id) {
    this.globalData.serverList = this.globalData.serverList.filter(s => s.id !== id)
    if (this.globalData.currentServerId === id) {
      const first = this.globalData.serverList[0]
      if (first) {
        this.switchServer(first.id)
      } else {
        this.globalData.currentServerId = null
        this.globalData.serverUrl = 'http://localhost:3000'
        this.globalData.authToken = ''
      }
    }
    this._saveServerList()
  },

  updateServer(id, data) {
    const server = this.globalData.serverList.find(s => s.id === id)
    if (server) {
      Object.assign(server, data)
      if (this.globalData.currentServerId === id) {
        this.globalData.serverUrl = server.url
        this.globalData.authToken = server.token
      }
      this._saveServerList()
    }
  },

  switchServer(id) {
    const server = this.globalData.serverList.find(s => s.id === id)
    if (!server) return
    this.globalData.currentServerId = id
    this.globalData.serverUrl = server.url
    this.globalData.authToken = server.token
    this.globalData.isConnected = false
    wx.setStorageSync('currentServerId', id)
    wx.setStorageSync('serverUrl', server.url)
    wx.setStorageSync('authToken', server.token)
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
