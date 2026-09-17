package me.siwi.irsclaw

import android.app.Application
import me.siwi.irsclaw.data.api.ClawApi
import me.siwi.irsclaw.data.settings.SettingsStore
import okhttp3.OkHttpClient

/** Manual dependency container — keeps the data layer framework-free like the iOS app. */
class AppContainer(context: android.content.Context) {
    val settingsStore: SettingsStore = SettingsStore(context)
    private val okHttpClient: OkHttpClient = OkHttpClient.Builder().build()
    val clawApi: ClawApi = ClawApi(settingsStore, okHttpClient)
}

class IrsClawApplication : Application() {
    lateinit var container: AppContainer
        private set

    override fun onCreate() {
        super.onCreate()
        container = AppContainer(this)
    }
}
