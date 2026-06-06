const DEFAULT_TIMEOUT = 15000
const STREAM_TIMEOUT = 60000
const RETRY_COUNT = 1

function baseUrl() {
  try {
    const app = getApp()
    const url = app && app.globalData && app.globalData.serverUrl
    if (!url) return null
    return String(url).replace(/\/+$/, '') + '/api'
  } catch (e) {
    return null
  }
}

function authHeader() {
  try {
    const app = getApp()
    const token = app && app.globalData && app.globalData.authToken
    if (!token) return {}
    return { Authorization: 'Bearer ' + token }
  } catch (e) {
    return {}
  }
}

function describeError(err) {
  const msg = (err && err.errMsg) || ''
  if (!msg) return '网络请求失败'
  if (/url/.test(msg)) return 'URL 错误: ' + msg
  if (/timeout/.test(msg)) return '请求超时'
  if (/fail/.test(msg)) return '连接失败,请检查服务器地址'
  if (/abort/.test(msg)) return '请求已取消'
  return msg
}

function request(method, path, data, options) {
  options = options || {}
  const timeout = options.timeout || DEFAULT_TIMEOUT
  const silent = options.silent === true

  return new Promise(function (resolve) {
    const url = baseUrl()
    if (!url) {
      if (!silent) {
        resolve({ success: false, error: '未配置服务器,请先在设置中添加服务器', code: 'NO_SERVER' })
      } else {
        resolve({ success: false, error: 'NO_SERVER' })
      }
      return
    }
    const doRequest = function (attempt) {
      wx.request({
        url: url + path,
        method: method,
        data: data,
        header: Object.assign({ 'Content-Type': 'application/json' }, authHeader()),
        timeout: timeout,
        success: function (res) {
          if (res.statusCode === 401) {
            resolve({ success: false, error: '认证失败,请检查 Auth Token', code: 'UNAUTHORIZED' })
            return
          }
          if (res.statusCode === 403) {
            resolve({ success: false, error: '没有权限', code: 'FORBIDDEN' })
            return
          }
          if (res.statusCode >= 500) {
            resolve({ success: false, error: '服务器错误 (HTTP ' + res.statusCode + ')', code: 'SERVER_ERROR' })
            return
          }
          if (res.statusCode >= 400) {
            const errMsg = (res.data && (res.data.error || res.data.message)) || ('请求失败 (HTTP ' + res.statusCode + ')')
            resolve({ success: false, error: errMsg, code: 'CLIENT_ERROR' })
            return
          }
          if (!res.data) {
            resolve({ success: false, error: '响应数据为空', code: 'EMPTY_RESPONSE' })
            return
          }
          resolve(res.data)
        },
        fail: function (err) {
          const isNetworkIssue = /fail|timeout/.test((err && err.errMsg) || '')
          if (isNetworkIssue && attempt < RETRY_COUNT) {
            setTimeout(function () { doRequest(attempt + 1) }, 600)
            return
          }
          resolve({ success: false, error: describeError(err), code: 'NETWORK_ERROR' })
        }
      })
    }
    doRequest(0)
  })
}

function healthCheck() {
  const url = baseUrl()
  if (!url) return Promise.resolve({ success: false, error: '未配置服务器' })
  return new Promise(function (resolve) {
    wx.request({
      url: url + '/health',
      method: 'GET',
      timeout: 5000,
      success: function (res) {
        if (res.statusCode >= 200 && res.statusCode < 300) {
          resolve({ success: true, data: res.data })
        } else {
          resolve({ success: false, error: '服务器响应异常 (HTTP ' + res.statusCode + ')' })
        }
      },
      fail: function () {
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
  const body = { positive: !!positive }
  if (message) body.message = message
  return request('POST', '/sessions/' + encodeURIComponent(sessionId) + '/feedback', body)
}

function sendMessage(message, agentId) {
  const body = { message: message }
  if (agentId && agentId !== 'default') {
    body.agent_id = agentId
  }
  return request('POST', '/chat', body)
}

/**
 * Send message and receive SSE stream directly from /api/chat (unified endpoint).
 * Returns a task object with abort() method.
 */
function sendMessageAndStream(message, agentId, handlers) {
  handlers = handlers || {}
  const url = baseUrl()
  const task = { aborted: false, _realTask: null, abort: function () { this.aborted = true; if (this._realTask && this._realTask.abort) { try { this._realTask.abort() } catch (e) {} } } }

  if (!url) {
    if (handlers.onError) handlers.onError({ error: '未配置服务器', code: 'NO_SERVER' })
    return task
  }

  const body = { message: message }
  if (agentId && agentId !== 'default') {
    body.agent_id = agentId
  }

  // SSE buffer accumulator: chunks may split a single SSE event across
  // multiple onChunkReceived callbacks, so we maintain a running buffer
  // and only parse complete (terminated by blank line) events.
  var buffer = ''
  var currentEvent = ''
  var currentData = ''
  var doneFired = false
  var extractedSessionId = null

  function processLine(line) {
    if (line.indexOf('event: ') === 0) {
      currentEvent = line.substring(7).trim()
    } else if (line.indexOf('data: ') === 0) {
      if (currentData) currentData += '\n'
      currentData += line.substring(6)
    } else if (line === '') {
      // blank line = event boundary
      if (currentEvent && currentData) {
        handleSseEvent(currentEvent, currentData, handlers)
        if (currentEvent === 'done') {
          doneFired = true
          // Extract session_id from done event data
          try {
            var doneData = JSON.parse(currentData)
            if (doneData.session_id) {
              extractedSessionId = doneData.session_id
            }
          } catch (e) {}
        }
      }
      currentEvent = ''
      currentData = ''
    }
  }

  function feedChunk(text) {
    buffer += text
    var lines = buffer.split('\n')
    // last element may be incomplete (no trailing newline), keep it in buffer
    buffer = lines.pop() || ''
    for (var i = 0; i < lines.length; i++) {
      processLine(lines[i].replace(/\r$/, ''))
    }
  }

  var realTask = wx.request({
    url: url + '/chat',
    method: 'POST',
    data: body,
    dataType: 'text',
    header: Object.assign({ 'Content-Type': 'application/json' }, authHeader()),
    timeout: 60000,
    success: function (res) {
      if (task.aborted) return
      // The SSE response may arrive either via onChunkReceived (if enableChunked
      // is supported) or as a complete body in res.data here. Drain both paths.
      if (res && res.statusCode === 401) {
        if (handlers.onError) handlers.onError({ error: '认证失败', code: 'UNAUTHORIZED' })
        return
      }
      if (res && res.statusCode >= 400) {
        if (handlers.onError) handlers.onError({ error: '请求失败 (HTTP ' + res.statusCode + ')', code: 'REQUEST_ERROR' })
        return
      }
      // Fallback: if data came through res.data directly (e.g., when enableChunked
      // doesn't work with POST), feed it through the same parser.
      // res.data may be a parsed JSON object (not SSE text) — only feed strings.
      if (res && res.data && typeof res.data === 'string') {
        feedChunk(res.data)
      }
      // Flush any trailing partial event that didn't end with a blank line
      if (buffer) {
        var trailing = buffer
        buffer = ''
        var tailLines = trailing.split('\n')
        for (var j = 0; j < tailLines.length; j++) {
          processLine(tailLines[j].replace(/\r$/, ''))
        }
        if (currentEvent && currentData) {
          handleSseEvent(currentEvent, currentData, handlers)
          if (currentEvent === 'done') {
            doneFired = true
            try {
              var doneDataTail = JSON.parse(currentData)
              if (doneDataTail.session_id) {
                extractedSessionId = doneDataTail.session_id
              }
            } catch (e) {}
          }
          currentEvent = ''
          currentData = ''
        }
      }
      if (!doneFired && handlers.onDone) {
        handlers.onDone(null, null, extractedSessionId)
      }
    },
    fail: function (err) {
      if (task.aborted) return
      if (handlers.onError) handlers.onError({ error: describeError(err), code: 'NETWORK_ERROR' })
    }
  })

  // Handle chunked response data
  try {
    realTask.onChunkReceived(function (res) {
      if (task.aborted) return
      if (!res || !res.data) return
      // onChunkReceived returns ArrayBuffer; decode UTF-8 to string
      try {
        var bytes = new Uint8Array(res.data)
        var text = ''
        for (var i = 0; i < bytes.length; i++) {
          text += String.fromCharCode(bytes[i])
        }
        try {
          text = decodeURIComponent(escape(text))
        } catch (e) {}
        if (text.length > 0) {
          feedChunk(text)
        }
      } catch (e) {}
    })
  } catch (e) {}

  task._realTask = realTask
  return task
}

function streamChat(sessionId, handlers) {
  handlers = handlers || {}
  const url = baseUrl()
  const task = { aborted: false, _realTask: null, abort: function () { this.aborted = true; if (this._realTask && this._realTask.abort) { try { this._realTask.abort() } catch (e) {} } } }

  if (!url) {
    if (handlers.onError) handlers.onError({ error: '未配置服务器', code: 'NO_SERVER' })
    return task
  }

  const streamUrl = url + '/chat/stream/' + encodeURIComponent(sessionId)

  // SSE buffer accumulator: chunks may split a single SSE event across
  // multiple onChunkReceived callbacks, so we maintain a running buffer
  // and only parse complete (terminated by blank line) events.
  var buffer = ''
  var currentEvent = ''
  var currentData = ''
  var receivedAny = false
  var doneFired = false

  function processLine(line) {
    if (line.indexOf('event: ') === 0) {
      currentEvent = line.substring(7).trim()
    } else if (line.indexOf('data: ') === 0) {
      if (currentData) currentData += '\n'
      currentData += line.substring(6)
    } else if (line === '') {
      // blank line = event boundary
      if (currentEvent && currentData) {
        receivedAny = true
        handleSseEvent(currentEvent, currentData, handlers)
        if (currentEvent === 'done') doneFired = true
      }
      currentEvent = ''
      currentData = ''
    }
  }

  function feedChunk(text) {
    buffer += text
    var lines = buffer.split('\n')
    // last element may be incomplete (no trailing newline), keep it in buffer
    buffer = lines.pop() || ''
    for (var i = 0; i < lines.length; i++) {
      processLine(lines[i].replace(/\r$/, ''))
    }
  }

  var realTask = wx.request({
    url: streamUrl,
    method: 'GET',
    header: authHeader(),
    enableChunked: true,
    timeout: STREAM_TIMEOUT,
    success: function (res) {
      if (task.aborted) return
      // With enableChunked:true the body comes via onChunkReceived, but
      // some platforms may still deliver a final payload here. Drain it
      // to be safe, then ensure onDone is fired exactly once.
      if (res && res.statusCode === 401) {
        if (handlers.onError) handlers.onError({ error: '认证失败', code: 'UNAUTHORIZED' })
        return
      }
      if (res && res.statusCode >= 400) {
        if (handlers.onError) handlers.onError({ error: '流式请求失败 (HTTP ' + res.statusCode + ')', code: 'STREAM_ERROR' })
        return
      }
      // Flush any trailing partial event that didn't end with a blank line
      if (buffer) {
        var trailing = buffer
        buffer = ''
        var tailLines = trailing.split('\n')
        for (var j = 0; j < tailLines.length; j++) {
          processLine(tailLines[j].replace(/\r$/, ''))
        }
        if (currentEvent && currentData) {
          handleSseEvent(currentEvent, currentData, handlers)
          if (currentEvent === 'done') doneFired = true
          currentEvent = ''
          currentData = ''
        }
      }
      if (!doneFired && handlers.onDone) {
        handlers.onDone(null, null)
      }
    },
    fail: function (err) {
      if (task.aborted) return
      if (handlers.onError) handlers.onError({ error: describeError(err), code: 'NETWORK_ERROR' })
    }
  })

  task._realTask = realTask

  // Primary data path: chunks arrive progressively as the LLM streams.
  try {
    realTask.onChunkReceived(function (res) {
      if (task.aborted) return
      if (!res || !res.data) return
      var bytes = new Uint8Array(res.data)
      var text = ''
      for (var i = 0; i < bytes.length; i++) {
        text += String.fromCharCode(bytes[i])
      }
      // bytes are UTF-8 — decode multi-byte sequences
      try {
        text = decodeURIComponent(escape(text))
      } catch (e) {}
      feedChunk(text)
    })
  } catch (e) {
    // Some runtimes don't expose onChunkReceived; fall back to success path
    console.warn('[SSE] onChunkReceived unavailable:', e)
  }

  return task
}

/** Process a single SSE event from streamChat */
function handleSseEvent(event, data, handlers) {
  try {
    switch (event) {
      case 'token':
        if (handlers.onToken) handlers.onToken(data)
        break
      case 'reasoning':
        if (handlers.onReasoning) handlers.onReasoning(data)
        break
      case 'status':
        if (handlers.onStatus) handlers.onStatus(data)
        break
      case 'new_round':
        if (handlers.onNewRound) handlers.onNewRound()
        break
      case 'tool_executed':
        if (handlers.onToolExecuted) {
          try {
            var info = JSON.parse(data)
            handlers.onToolExecuted(info)
          } catch (e) {
            handlers.onToolExecuted({ name: data, args: '', result: '' })
          }
        }
        break
      case 'evaluation':
        if (handlers.onEvaluation) {
          try {
            var evalInfo = JSON.parse(data)
            handlers.onEvaluation(evalInfo)
          } catch (e) {}
        }
        break
      case 'image_generated':
        if (handlers.onImageGenerated) {
          try {
            var imgInfo = JSON.parse(data)
            handlers.onImageGenerated(imgInfo)
          } catch (e) {}
        }
        break
      case 'quality_score':
        if (handlers.onQuality) {
          try {
            var qualityInfo = JSON.parse(data)
            handlers.onQuality(qualityInfo)
          } catch (e) {}
        }
        break
      case 'done':
        if (handlers.onDone) {
          try {
            var doneData = JSON.parse(data)
            var usage = doneData.usage || null
            var quality = doneData.quality || null
            var sessionId = doneData.session_id || null
            handlers.onDone(usage, quality, sessionId)
          } catch (e) {
            handlers.onDone(null, null, null)
          }
        }
        break
      case 'error':
        if (handlers.onError) handlers.onError({ error: data, code: 'LLM_ERROR' })
        break
      default:
        break
    }
  } catch (e) {
    console.error('[SSE] handler error:', e)
  }
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
  sendMessageAndStream: sendMessageAndStream,
  streamChat: streamChat,
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
