var app = getApp()

function baseUrl() {
  return app.globalData.serverUrl + '/api'
}

function authHeader() {
  var token = app.globalData.authToken
  if (!token) return {}
  return { Authorization: 'Bearer ' + token }
}

function request(method, path, data) {
  return new Promise(function(resolve, reject) {
    wx.request({
      url: baseUrl() + path,
      method: method,
      data: data,
      header: Object.assign({ 'Content-Type': 'application/json' }, authHeader()),
      success: function(res) {
        resolve(res.data)
      },
      fail: function(err) {
        reject(err)
      }
    })
  })
}

function healthCheck() {
  return request('GET', '/health')
}

function getConfig() {
  return request('GET', '/config')
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

function streamChat(sessionId, handlers) {
  var url = baseUrl() + '/chat/stream/' + encodeURIComponent(sessionId)
  var task = wx.request({
    url: url,
    method: 'GET',
    header: Object.assign({}, authHeader()),
    enableChunked: true,
    success: function() {},
    fail: function(err) {
      if (handlers.onError) handlers.onError(String(err))
    }
  })

  task.onChunkReceived(function(res) {
    var bytes = new Uint8Array(res.data)
    var text = ''
    for (var i = 0; i < bytes.length; i++) {
      text += String.fromCharCode(bytes[i])
    }
    try {
      text = decodeURIComponent(escape(text))
    } catch (e) {}

    var lines = text.split('\n')
    var currentEvent = ''
    for (var j = 0; j < lines.length; j++) {
      var line = lines[j].trim()
      if (line.indexOf('event: ') === 0) {
        currentEvent = line.slice(7).trim()
      } else if (line.indexOf('data: ') === 0) {
        var data = line.slice(6)
        if (currentEvent === 'token' && handlers.onToken) {
          handlers.onToken(data)
        } else if (currentEvent === 'reasoning' && handlers.onReasoning) {
          handlers.onReasoning(data)
        } else if (currentEvent === 'status' && handlers.onStatus) {
          handlers.onStatus(data)
        } else if (currentEvent === 'error' && handlers.onError) {
          handlers.onError(data)
        } else if (currentEvent === 'new_round' && handlers.onNewRound) {
          handlers.onNewRound()
        } else if (currentEvent === 'tool_executed' && handlers.onToolExecuted) {
          try {
            var parsed = JSON.parse(data)
            handlers.onToolExecuted(parsed)
          } catch (e) {}
        } else if (currentEvent === 'done' && handlers.onDone) {
          try {
            var doneParsed = JSON.parse(data)
            handlers.onDone(doneParsed.usage)
          } catch (e) {
            handlers.onDone(null)
          }
        }
      }
    }
  })

  return task
}

module.exports = {
  healthCheck: healthCheck,
  getConfig: getConfig,
  listSessions: listSessions,
  getCurrentSession: getCurrentSession,
  createSession: createSession,
  switchSession: switchSession,
  getSession: getSession,
  deleteSession: deleteSession,
  sendMessage: sendMessage,
  listTools: listTools,
  listPlugins: listPlugins,
  listSkills: listSkills,
  listAgents: listAgents,
  getAgentConfig: getAgentConfig,
  createAgent: createAgent,
  updateAgent: updateAgent,
  deleteAgent: deleteAgent,
  streamChat: streamChat
}