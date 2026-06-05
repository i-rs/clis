const store = require('../../utils/store.js')
const app = getApp()

Page({
  data: {
    isConnected: false,
    displayUrl: '',
    servers: [],
    currentServerId: '',
    showEditSheet: false,
    editingServer: null,
    editName: '',
    editUrl: '',
    editToken: '',
    showToken: false
  },

  onLoad: function () {
    const that = this
    // 订阅 store 状态变化，自动同步 UI
    this._storeSubId = store.subscribe(function (s) {
      that.setData({
        isConnected: s.isConnected,
        displayUrl: s.displayUrl,
        servers: s.servers,
        currentServerId: s.currentServerId
      })
    })
    // 初始化状态
    store.loadFromStorage()
  },

  onUnload: function () {
    if (this._storeSubId) {
      store.unsubscribe(this._storeSubId)
    }
  },

  goBack: function () { wx.navigateBack() },

  showAddSheet: function () {
    this.setData({
      showEditSheet: true,
      editingServer: null,
      editName: '',
      editUrl: '',
      editToken: '',
      showToken: false
    })
  },

  hideEditSheet: function () {
    this.setData({ showEditSheet: false })
  },

  preventMaskTap: function () {},

  onEditName: function (e) { this.setData({ editName: e.detail.value }) },
  onEditUrl: function (e) { this.setData({ editUrl: e.detail.value }) },
  onEditToken: function (e) { this.setData({ editToken: e.detail.value }) },

  onShowTokenChange: function (e) {
    this.setData({ showToken: e.detail.value.includes('show') })
  },

  selectServer: function (e) {
    store.activateServer(e.currentTarget.dataset.id)
  },

  editServer: function (e) {
    const id = e.currentTarget.dataset.id
    const servers = this.data.servers
    for (let i = 0; i < servers.length; i++) {
      if (servers[i].id === id) {
        const srv = servers[i]
        this.setData({
          showEditSheet: true,
          editingServer: srv,
          editName: srv.name,
          editUrl: srv.url,
          editToken: srv.token || '',
          showToken: false
        })
        return
      }
    }
  },

  saveEdit: function () {
    const name = this.data.editName.trim()
    const url = this.data.editUrl.trim()
    if (!name || !url) {
      wx.showToast({ title: '请填写名称和地址', icon: 'none' })
      return
    }
    if (!store.isValidUrl(url)) {
      wx.showToast({ title: '地址格式不正确,需以 http/https 开头', icon: 'none' })
      return
    }

    const token = this.data.editToken.trim()
    const editingServer = this.data.editingServer

    if (editingServer) {
      // 更新
      store.upsertServer({
        id: editingServer.id,
        name: name,
        url: url,
        token: token,
        tokenDisplay: store.maskToken(token)
      })
      store.activateServer(editingServer.id)
      wx.showToast({ title: '已更新', icon: 'success' })
    } else {
      // 新增
      const id = store.genId('srv')
      store.upsertServer({
        id: id,
        name: name,
        url: url,
        token: token,
        tokenDisplay: store.maskToken(token)
      })
      store.activateServer(id)
      wx.showToast({ title: '已添加并连接', icon: 'success' })
    }
    this.setData({ showEditSheet: false })
  },

  deleteServer: function (e) {
    const id = e.currentTarget.dataset.id
    const name = e.currentTarget.dataset.name
    const that = this

    wx.showModal({
      title: '删除服务器',
      content: '确认删除「' + name + '」?',
      success: function (res) {
        if (!res.confirm) return
        store.deleteServer(id)
        wx.showToast({ title: '已删除', icon: 'success' })
      }
    })
  }
})
