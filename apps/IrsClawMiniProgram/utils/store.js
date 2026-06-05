/**
 * 全局状态管理 - 统一管理应用状态
 * 所有页面共享同一份状态，状态变化时自动通知所有订阅者
 */

const app = getApp()

// 状态数据
let state = {
  isConnected: false,
  displayUrl: '',
  serverUrl: '',
  authToken: '',
  servers: [],
  currentServerId: '',
  currentServerName: '',
  theme: 'dark',
  currentAgent: 'default',
  aiConfig: null
}

// 订阅者列表
const subscribers = {}

// 生成唯一订阅者ID
let subIdCounter = 0

/**
 * 订阅状态变化
 * @param {Function} callback - 状态变化时的回调函数，接收完整 state
 * @returns {string} subscriptionId - 取消订阅需要用到
 */
function subscribe(callback) {
  const id = 'sub_' + (++subIdCounter)
  subscribers[id] = callback
  // 立即同步当前状态
  callback(state)
  return id
}

/**
 * 取消订阅
 * @param {string} subscriptionId - subscribe 返回的ID
 */
function unsubscribe(subscriptionId) {
  delete subscribers[subscriptionId]
}

/**
 * 通知所有订阅者状态变化
 */
function notifySubscribers() {
  for (const id in subscribers) {
    if (typeof subscribers[id] === 'function') {
      try {
        subscribers[id](state)
      } catch (e) {
        console.error('[Store] subscriber error:', e)
      }
    }
  }
}

/**
 * 更新状态
 * @param {Object} partial - 要更新的部分状态
 */
function setState(partial) {
  const changed = {}
  for (const key in partial) {
    if (state[key] !== partial[key]) {
      changed[key] = partial[key]
      state[key] = partial[key]
    }
  }
  if (Object.keys(changed).length > 0) {
    notifySubscribers()
  }
}

/**
 * 获取完整状态
 */
function getState() {
  return state
}

/**
 * 从 storage 加载状态
 */
function loadFromStorage() {
  const servers = wx.getStorageSync('claw_servers') || []
  const currentId = wx.getStorageSync('claw_current_server') || ''

  // 规范化 servers
  for (let i = 0; i < servers.length; i++) {
    servers[i].tokenDisplay = maskToken(servers[i].token || '')
  }

  let currentServerName = ''
  for (let i = 0; i < servers.length; i++) {
    if (servers[i].id === currentId) {
      currentServerName = servers[i].name
      break
    }
  }

  const g = app.globalData
  setState({
    servers: servers,
    currentServerId: currentId,
    currentServerName: currentServerName,
    serverUrl: g.serverUrl || '',
    authToken: g.authToken || '',
    displayUrl: g.serverUrl || '',
    theme: g.theme || 'dark',
    currentAgent: g.currentAgent || 'default'
  })
}

/**
 * 激活服务器
 */
function activateServer(id) {
  const servers = state.servers
  for (let i = 0; i < servers.length; i++) {
    if (servers[i].id === id) {
      const srv = servers[i]
      app.globalData.serverUrl = srv.url
      app.globalData.authToken = srv.token || ''
      app.saveConfig()
      wx.setStorageSync('claw_current_server', id)
      setState({
        currentServerId: id,
        currentServerName: srv.name,
        serverUrl: srv.url,
        authToken: srv.token || '',
        displayUrl: srv.url
      })
      app.notifyServerChanged()
      return
    }
  }
}

/**
 * 保存服务器列表
 */
function saveServers(servers) {
  wx.setStorageSync('claw_servers', servers)
  setState({ servers: servers })
}

/**
 * 添加或更新服务器
 */
function upsertServer(serverData) {
  const servers = state.servers.slice()
  let isNew = true
  for (let i = 0; i < servers.length; i++) {
    if (servers[i].id === serverData.id) {
      servers[i] = { ...servers[i], ...serverData }
      isNew = false
      break
    }
  }
  if (isNew) {
    servers.push(serverData)
  }
  saveServers(servers)
  return isNew ? servers[servers.length - 1].id : serverData.id
}

/**
 * 删除服务器
 */
function deleteServer(id) {
  const servers = state.servers.filter(s => s.id !== id)
  saveServers(servers)
  if (state.currentServerId === id) {
    if (servers.length > 0) {
      activateServer(servers[0].id)
    } else {
      app.globalData.serverUrl = ''
      app.globalData.authToken = ''
      app.saveConfig()
      wx.removeStorageSync('claw_current_server')
      setState({
        currentServerId: '',
        currentServerName: '',
        serverUrl: '',
        authToken: '',
        displayUrl: ''
      })
      app.notifyServerChanged()
    }
  }
}

/**
 * 隐藏令牌
 */
function maskToken(token) {
  if (!token) return ''
  if (token.length <= 8) return '••••'
  return token.slice(0, 4) + '••••••••' + token.slice(-4)
}

/**
 * 验证URL格式
 */
function isValidUrl(url) {
  return typeof url === 'string' && (url.startsWith('http://') || url.startsWith('https://'))
}

/**
 * 生成唯一ID
 */
function genId(prefix) {
  return prefix + '_' + Date.now() + '_' + Math.random().toString(36).substr(2, 9)
}

/**
 * 更新连接状态
 */
function updateConnectionStatus(isConnected) {
  setState({ isConnected })
}

module.exports = {
  subscribe,
  unsubscribe,
  setState,
  getState,
  loadFromStorage,
  activateServer,
  saveServers,
  upsertServer,
  deleteServer,
  updateConnectionStatus,
  maskToken,
  isValidUrl,
  genId
}
