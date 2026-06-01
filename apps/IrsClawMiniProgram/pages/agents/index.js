var api = require('../../utils/api.js')
var app = getApp()

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

  onLoad: function() {
    this.setData({ currentAgent: app.globalData.currentAgent || 'default' })
    this.loadAgents()
  },

  goBack: function() {
    wx.navigateBack()
  },

  loadAgents: function() {
    var that = this
    api.listAgents().then(function(res) {
      if (res.success && res.data) {
        var userAgents = res.data.filter(function(a) { return !a.is_sub_agent })
        that.setData({ agents: userAgents, loaded: true })
      } else {
        that.setData({ agents: [], loaded: true })
      }
    }).catch(function() {
      that.setData({ agents: [], loaded: true })
    })
  },

  onSwitchAgent: function(e) {
    var id = e.currentTarget.dataset.id
    app.globalData.currentAgent = id
    app.saveConfig()
    this.setData({ currentAgent: id })
    wx.showToast({ title: '已切换', icon: 'success' })
  },

  onShowAgentDetail: function(e) {
    var that = this
    var id = e.currentTarget.dataset.id
    api.getAgentConfig(id).then(function(res) {
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

  onAddAgent: function() {
    this.setData({
      showingForm: true,
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
    })
  },

  onCloseForm: function() {
    this.setData({ showingForm: false })
  },

  onFormInput: function(e) {
    var field = e.currentTarget.dataset.field
    var value = e.detail.value
    var formData = this.data.formData
    formData[field] = value
    this.setData({ formData: formData })
  },

  onSubmitForm: function() {
    var that = this
    var formData = this.data.formData
    if (!formData.id) {
      wx.showToast({ title: '请输入 Agent ID', icon: 'none' })
      return
    }
    var body = {}
    if (formData.provider) body.provider = formData.provider
    if (formData.model) body.model = formData.model
    if (formData.api_key) body.api_key = formData.api_key
    if (formData.base_url) body.base_url = formData.base_url
    if (formData.system_prompt) body.system_prompt = formData.system_prompt

    var promise
    if (this.data.formMode === 'create') {
      body.id = formData.id
      promise = api.createAgent(body)
    } else {
      promise = api.updateAgent(formData.id, body)
    }

    promise.then(function(res) {
      if (res.success) {
        wx.showToast({ title: '保存成功', icon: 'success' })
        that.setData({ showingForm: false })
        that.loadAgents()
      } else {
        wx.showToast({ title: res.error || '保存失败', icon: 'none' })
      }
    })
  },

  onDeleteAgent: function() {
    var that = this
    var id = this.data.editingAgent.id
    if (id === 'default') {
      wx.showToast({ title: '默认 Agent 不能删除', icon: 'none' })
      return
    }
    wx.showModal({
      title: '确认删除',
      content: '确定要删除 Agent ' + id + ' 吗？',
      success: function(res) {
        if (res.confirm) {
          api.deleteAgent(id).then(function(res) {
            if (res.success) {
              wx.showToast({ title: '已删除', icon: 'success' })
              that.setData({ showingForm: false })
              that.loadAgents()
            } else {
              wx.showToast({ title: res.error || '删除失败', icon: 'none' })
            }
          })
        }
      }
    })
  }
})