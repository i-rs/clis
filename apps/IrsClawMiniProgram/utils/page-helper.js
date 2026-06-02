function bindServerWatcher(page, onChanged) {
  const app = getApp()
  page._lastServerUrl = app.globalData.serverUrl || ''
  page._lastAuthToken = app.globalData.authToken || ''

  page.checkServerChanged = function () {
    const curUrl = app.globalData.serverUrl || ''
    const curToken = app.globalData.authToken || ''
    if (this._lastServerUrl !== curUrl || this._lastAuthToken !== curToken) {
      this._lastServerUrl = curUrl
      this._lastAuthToken = curToken
      onChanged.call(this)
    }
  }
}

function formatDate(dateStr) {
  if (!dateStr) return ''
  const d = new Date(dateStr)
  if (isNaN(d.getTime())) return String(dateStr).substring(0, 10)
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const yesterday = new Date(today.getTime() - 86400000)
  const weekStart = new Date(today.getTime() - today.getDay() * 86400000)

  if (d >= today) {
    const h = d.getHours()
    const m = d.getMinutes()
    return (h < 10 ? '0' : '') + h + ':' + (m < 10 ? '0' : '') + m
  }
  if (d >= yesterday) return '昨天'
  if (d >= weekStart) {
    const days = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']
    return days[d.getDay()]
  }
  return (d.getMonth() + 1) + '/' + d.getDate()
}

function dateGroup(dateStr) {
  if (!dateStr) return '更早'
  const d = new Date(dateStr)
  if (isNaN(d.getTime())) return '更早'
  const now = new Date()
  const today = new Date(now.getFullYear(), now.getMonth(), now.getDate())
  const yesterday = new Date(today.getTime() - 86400000)
  const weekStart = new Date(today.getTime() - today.getDay() * 86400000)

  if (d >= today) return '今天'
  if (d >= yesterday) return '昨天'
  if (d >= weekStart) return '本周'
  return '更早'
}

function groupByDate(items, dateField) {
  const order = ['今天', '昨天', '本周', '更早']
  const buckets = {}
  for (let i = 0; i < order.length; i++) buckets[order[i]] = []

  for (let i = 0; i < items.length; i++) {
    const g = dateGroup(items[i][dateField])
    if (buckets[g]) buckets[g].push(items[i])
  }

  const result = []
  for (let i = 0; i < order.length; i++) {
    if (buckets[order[i]].length > 0) {
      result.push({ name: order[i], items: buckets[order[i]] })
    }
  }
  return result
}

function toSingleLine(text) {
  if (!text) return ''
  return String(text).replace(/\s+/g, ' ').trim()
}

function smartTruncate(text, maxLen) {
  if (!text) return ''
  if (text.length <= maxLen) return text
  return text.substring(0, maxLen) + '…'
}

function formatJson(text) {
  if (!text) return ''
  const str = typeof text === 'string' ? text : String(text)
  try {
    const obj = JSON.parse(str)
    if (typeof obj === 'object' && obj !== null) {
      return JSON.stringify(obj, null, 2)
    }
  } catch (e) {}
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
  const count = text.length
  if (count < 1000) return count + 'c'
  return Math.floor(count / 1000) + 'k'
}

function maskToken(token) {
  if (!token) return ''
  if (token.length <= 8) return '••••••••'
  return token.substring(0, 4) + '••••••••' + token.substring(token.length - 4)
}

function formatNumber(n) {
  if (n === null || n === undefined || n === '') return '0'
  return String(n).replace(/\B(?=(\d{3})+(?!\d))/g, ',')
}

function formatCost(cost) {
  if (cost === null || cost === undefined || cost === '') return '$0.00'
  const n = Number(cost)
  if (isNaN(n)) return '$0.00'
  return '$' + n.toFixed(4)
}

function debounce(fn, delay) {
  let timer = null
  return function () {
    const args = arguments
    const ctx = this
    if (timer) clearTimeout(timer)
    timer = setTimeout(function () { fn.apply(ctx, args) }, delay)
  }
}

function throttle(fn, gap) {
  let last = 0
  return function () {
    const now = Date.now()
    if (now - last >= gap) {
      last = now
      fn.apply(this, arguments)
    }
  }
}

function isValidUrl(url) {
  if (!url) return false
  return /^https?:\/\/.+/i.test(String(url).trim())
}

function genId(prefix) {
  return (prefix || 'id') + '_' + Date.now() + '_' + Math.floor(Math.random() * 10000)
}

module.exports = {
  bindServerWatcher: bindServerWatcher,
  formatDate: formatDate,
  dateGroup: dateGroup,
  groupByDate: groupByDate,
  toSingleLine: toSingleLine,
  smartTruncate: smartTruncate,
  formatJson: formatJson,
  charCount: charCount,
  maskToken: maskToken,
  formatNumber: formatNumber,
  formatCost: formatCost,
  debounce: debounce,
  throttle: throttle,
  isValidUrl: isValidUrl,
  genId: genId
}
