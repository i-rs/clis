const app = getApp()

Page({
  data: {
    skills: []
  },

  onLoad() {
    this.loadSkills()
  },

  onShow() {
    this.loadSkills()
  },

  async loadSkills() {
    const skills = await app.getSkills()
    this.setData({ skills })
  }
})
