function baseUrl() {
  try {
    var app = getApp()
    var url = app && app.globalData && app.globalData.serverUrl
    if (!url) return null
    return url.replace(/\/+$/, '') + '/api'
  } catch (e) {
    return null
  }
}

function authHeader() {
  try {
    var app = getApp()
    var token = app && app.globalData && app.globalData.authToken
    if (!token) return {}
    return { Authorization: 'Bearer ' + token }
  } catch (e) {
    return {}
  }
}

function request(method, path, data) {
  return new Promise(function(resolve, reject) {
    var url = baseUrl()
    if (!url) {
      resolve({ success: false, error: '未配置服务器，请先在设置中添加服务器', code: 'NO_SERVER' })
      return
    }
    wx.request({
      url: url + path,
      method: method,
      data: data,
      header: Object.assign({ 'Content-Type': 'application/json' }, authHeader()),
      success: function(res) {
        if (res.statusCode === 401) {
          resolve({ success: false, error: '认证失败，请检查 Auth Token', code: 'UNAUTHORIZED' })
          return
        }
        if (res.statusCode >= 500) {
          resolve({ success: false, error: '服务器错误 (HTTP ' + res.statusCode + ')', code: 'SERVER_ERROR' })
          return
        }
        if (!res.data) {
          resolve({ success: false, error: '响应数据为空', code: 'EMPTY_RESPONSE' })
          return
        }
        resolve(res.data)
      },
      fail: function(err) {
        var msg = '网络请求失败'
        if (err && err.errMsg) {
          if (err.errMsg.indexOf('url') !== -1) msg = 'URL 错误: ' + err.errMsg
          else if (err.errMsg.indexOf('timeout') !== -1) msg = '请求超时'
          else if (err.errMsg.indexOf('fail') !== -1) msg = '连接失败，请检查服务器地址'
        }
        resolve({ success: false, error: msg, code: 'NETWORK_ERROR' })
      }
    })
  })
}

function healthCheck() {
  var url = baseUrl()
  if (!url) return Promise.resolve({ success: false, error: '未配置服务器' })
  return new Promise(function(resolve) {
    wx.request({
      url: url + '/health',
      method: 'GET',
      timeout: 5000,
      success: function(res) {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          resolve({ success: true, data: res.data })
        } else {
          resolve({ success: false, error: '服务器响应异常 (HTTP ' + res.statusCode + ')' })
        }
      },
      fail: function() {
        resolve({ success: false, error: '无法连接到服务器' })
      }
    })
  })
}

function getConfig() {
  return request('GET', '/config')
}

function updateConfig(body) {
  return request('PUT', '/config', body)
}

function listSessions() {
  return request('GET', '/sessions')
}

function getCurrentSession() {
  return request('GET', '/sessions/current')
}

function createSession(agentId) {
  return request('POST', '/sessions', agentId ? { agent_id: agentId } : {})
}

function switchSession(id) {
  return request('POST', '/sessions/' + encodeURIComponent(id) + '/switch')
}

function getSession(id) {
  return request('GET', '/sessions/' + encodeURIComponent(id))
}

function deleteSession(id) {
  return request('DELETE', '/sessions/' + encodeURIComponent(id))
}

function postFeedback(sessionId, positive, message) {
  var body = { positive: positive }
  if (message) body.message = message
  return request('POST', '/sessions/' + encodeURIComponent(sessionId) + '/feedback', body)
}

function sendMessage(message, agentId) {
  var body = { message: message }
  if (agentId && agentId !== 'default') {
    body.agent_id = agentId
  }
  return request('POST', '/chat', body)
}

function listTools() {
  return request('GET', '/tools')
}

function listPlugins() {
  return request('GET', '/plugins')
}

function listSkills() {
  return request('GET', '/skills')
}

function listAgents() {
  return request('GET', '/agents')
}

function getAgentConfig(id) {
  return request('GET', '/agents/' + encodeURIComponent(id))
}

function createAgent(body) {
  return request('POST', '/agents', body)
}

function updateAgent(id, body) {
  return request('PUT', '/agents/' + encodeURIComponent(id), body)
}

function deleteAgent(id) {
  return request('DELETE', '/agents/' + encodeURIComponent(id))
}

function getStats(period) {
  return request('GET', '/stats?period=' + encodeURIComponent(period || 'all'))
}

module.exports = {
  healthCheck: healthCheck,
  getConfig: getConfig,
  updateConfig: updateConfig,
  listSessions: listSessions,
  getCurrentSession: getCurrentSession,
  createSession: createSession,
  switchSession: switchSession,
  getSession: getSession,
  deleteSession: deleteSession,
  postFeedback: postFeedback,
  sendMessage: sendMessage,
  listTools: listTools,
  listPlugins: listPlugins,
  listSkills: listSkills,
  listAgents: listAgents,
  getAgentConfig: getAgentConfig,
  createAgent: createAgent,
  updateAgent: updateAgent,
  deleteAgent: deleteAgent,
  getStats: getStats
}