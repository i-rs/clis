const api = require('../../utils/api.js')
const helper = require('../../utils/page-helper.js')
const app = getApp()

Page({
  data: {
    agents: [],
    currentAgent: 'default',
    loaded: false,
    showingForm: false,
    formMode: 'create',
    editingAgent: null,
    formData: {
      id: '',
      provider: '',
      model: '',
      api_key: '',
      base_url: '',
      system_prompt: ''
    }
  },

  onLoad: function () {
    helper.bindServerWatcher(this, function () { this.onServerChanged() })
    this.setData({ currentAgent: app.globalData.currentAgent || 'default' })
    this.loadAgents()
  },

  onShow: function () {
    this.checkServerChanged()
    this.setData({ currentAgent: app.globalData.currentAgent || 'default' })
    this.loadAgents()
  },

  onServerChanged: function () {
    this.setData({ agents: [] })
    this.loadAgents()
  },

  goBack: function () {
    wx.navigateBack()
  },

  loadAgents: function () {
    const that = this
    api.listAgents().then(function (res) {
      if (res.success && res.data) {
        const userAgents = res.data.filter(function (a) { return !a.is_sub_agent })
        that.setData({ agents: userAgents, loaded: true })
      } else {
        that.setData({ agents: [], loaded: true })
      }
    }).catch(function () {
      that.setData({ agents: [], loaded: true })
    })
  },

  onSwitchAgent: function (e) {
    const id = e.currentTarget.dataset.id
    if (id === app.globalData.currentAgent) return
    app.globalData.currentAgent = id
    app.saveConfig()
    app.globalData.sessionId = null
    this.setData({ currentAgent: id })
    app.notifyServerChanged()
    wx.showToast({ title: '已切换', icon: 'success' })
  },

  onShowAgentDetail: function (e) {
    const that = this
    const id = e.currentTarget.dataset.id
    api.getAgentConfig(id).then(function (res) {
      if (res.success && res.data) {
        that.setData({
          showingForm: true,
          formMode: 'edit',
          editingAgent: res.data,
          formData: {
            id: res.data.id,
            provider: res.data.provider || '',
            model: res.data.model || '',
            api_key: res.data.api_key || '',
            base_url: res.data.base_url || '',
            system_prompt: res.data.system_prompt || ''
          }
        })
      } else {
        wx.showToast({ title: '获取详情失败', icon: 'none' })
      }
    })
  },

  onAddAgent: function () {
    this.setData({
      showingForm: true,
      formMode: 'create',
      editingAgent: null,
      formData: { id: '', provider: '', model: '', api_key: '', base_url: '', system_prompt: '' }
    })
  },

  onCloseForm: function () {
    this.setData({ showingForm: false })
  },

  onFormInput: function (e) {
    const field = e.currentTarget.dataset.field
    const value = e.detail.value
    const formData = Object.assign({}, this.data.formData)
    formData[field] = value
    this.setData({ formData: formData })
  },

  onSubmitForm: function () {
    const that = this
    const formData = this.data.formData
    if (!formData.id) {
      wx.showToast({ title: '请输入 Agent ID', icon: 'none' })
      return
    }
    if (!/^[a-zA-Z0-9_-]+$/.test(formData.id)) {
      wx.showToast({ title: 'ID 只能包含字母数字下划线', icon: 'none' })
      return
    }
    const body = {}
    if (formData.provider) body.provider = formData.provider
    if (formData.model) body.model = formData.model
    if (formData.api_key) body.api_key = formData.api_key
    if (formData.base_url) body.base_url = formData.base_url
    if (formData.system_prompt) body.system_prompt = formData.system_prompt

    const promise = this.data.formMode === 'create'
      ? (body.id = formData.id, api.createAgent(body))
      : api.updateAgent(formData.id, body)

    promise.then(function (res) {
      if (res.success) {
        wx.showToast({ title: '保存成功', icon: 'success' })
        that.setData({ showingForm: false })
        that.loadAgents()
      } else {
        wx.showToast({ title: res.error || '保存失败', icon: 'none' })
      }
    })
  },

  onDeleteAgent: function () {
    const that = this
    const id = this.data.editingAgent && this.data.editingAgent.id
    if (!id || id === 'default') {
      wx.showToast({ title: '默认 Agent 不能删除', icon: 'none' })
      return
    }
    wx.showModal({
      title: '确认删除',
      content: '确定要删除 Agent ' + id + ' 吗?',
      success: function (res) {
        if (res.confirm) {
          api.deleteAgent(id).then(function (r) {
            if (r.success) {
              wx.showToast({ title: '已删除', icon: 'success' })
              that.setData({ showingForm: false })
              that.loadAgents()
            } else {
              wx.showToast({ title: r.error || '删除失败', icon: 'none' })
            }
          })
        }
      }
    })
  }
})
