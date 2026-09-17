package me.siwi.irsclaw.data.settings

import android.content.Context
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import me.siwi.irsclaw.data.model.TokenUsage
import java.util.UUID

private val Context.dataStore by preferencesDataStore(name = "settings")

/** One named claw-serve backend, mirroring the iOS `BackendConfig`. */
@Serializable
data class BackendConfig(
    val id: String = UUID.randomUUID().toString(),
    val name: String = "Local",
    val url: String = "http://127.0.0.1:3000",
    val authToken: String = "",
)

enum class Appearance(val label: String) {
    LIGHT("浅色"), DARK("深色"), AUTO("自动");

    companion object {
        fun from(raw: String?): Appearance = entries.firstOrNull { it.name.equals(raw, true) } ?: AUTO
    }
}

data class AppSettings(
    val backends: List<BackendConfig>,
    val currentBackend: BackendConfig,
    val llmProvider: String,
    val llmApiKey: String,
    val llmBaseUrl: String,
    val appearance: Appearance,
    val sessionTokenUsage: Map<String, TokenUsage>,
)

/**
 * All persisted app state. Preference keys intentionally mirror the iOS client's
 * UserDefaults keys (`backend_configs`, `current_backend_id`, `llm_*`,
 * `app_appearance`, `session_token_usage_v2`).
 */
class SettingsStore(private val context: Context) {

    private object Keys {
        val BACKEND_CONFIGS = stringPreferencesKey("backend_configs")
        val CURRENT_BACKEND_ID = stringPreferencesKey("current_backend_id")
        val LLM_PROVIDER = stringPreferencesKey("llm_provider")
        val LLM_API_KEY = stringPreferencesKey("llm_api_key")
        val LLM_BASE_URL = stringPreferencesKey("llm_base_url")
        val APP_APPEARANCE = stringPreferencesKey("app_appearance")
        val SESSION_TOKEN_USAGE = stringPreferencesKey("session_token_usage_v2")
    }

    private val json = Json { ignoreUnknownKeys = true }

    val settings: Flow<AppSettings> = context.dataStore.data.map { prefs -> prefs.toSettings() }

    /** Resolves the active backend, creating a default profile on first launch. */
    suspend fun currentBackend(): BackendConfig = context.dataStore.data.first().toSettings().currentBackend

    suspend fun addBackend(config: BackendConfig) = updateBackends { it + config }

    suspend fun updateBackend(config: BackendConfig) = updateBackends { list ->
        list.map { if (it.id == config.id) config else it }
    }

    suspend fun deleteBackend(id: String) = updateBackends { list -> list.filterNot { it.id == id } }

    suspend fun setCurrentBackend(id: String) {
        context.dataStore.edit { it[Keys.CURRENT_BACKEND_ID] = id }
    }

    suspend fun setLlmConfig(provider: String, apiKey: String, baseUrl: String) {
        context.dataStore.edit {
            it[Keys.LLM_PROVIDER] = provider
            it[Keys.LLM_API_KEY] = apiKey
            it[Keys.LLM_BASE_URL] = baseUrl
        }
    }

    suspend fun setAppearance(appearance: Appearance) {
        context.dataStore.edit { it[Keys.APP_APPEARANCE] = appearance.name }
    }

    suspend fun addSessionUsage(sessionId: String, usage: TokenUsage) {
        context.dataStore.edit { prefs ->
            val existing = decodeUsage(prefs[Keys.SESSION_TOKEN_USAGE])
            val updated = (existing[sessionId] ?: TokenUsage()) + usage
            prefs[Keys.SESSION_TOKEN_USAGE] = json.encodeToString<Map<String, TokenUsage>>(
                existing + (sessionId to updated),
            )
        }
    }

    private suspend fun updateBackends(transform: (List<BackendConfig>) -> List<BackendConfig>) {
        context.dataStore.edit { prefs ->
            val list = decodeBackends(prefs[Keys.BACKEND_CONFIGS])
            prefs[Keys.BACKEND_CONFIGS] = json.encodeToString(transform(list))
        }
    }

    private fun androidx.datastore.preferences.core.Preferences.toSettings(): AppSettings {
        val backends = decodeBackends(this[Keys.BACKEND_CONFIGS])
        val currentId = this[Keys.CURRENT_BACKEND_ID]
        return AppSettings(
            backends = backends,
            currentBackend = backends.firstOrNull { it.id == currentId } ?: backends.first(),
            llmProvider = this[Keys.LLM_PROVIDER] ?: "",
            llmApiKey = this[Keys.LLM_API_KEY] ?: "",
            llmBaseUrl = this[Keys.LLM_BASE_URL] ?: "",
            appearance = Appearance.from(this[Keys.APP_APPEARANCE]),
            sessionTokenUsage = decodeUsage(this[Keys.SESSION_TOKEN_USAGE]),
        )
    }

    private fun decodeBackends(raw: String?): List<BackendConfig> {
        if (raw.isNullOrBlank()) return listOf(BackendConfig())
        return runCatching { json.decodeFromString<List<BackendConfig>>(raw) }.getOrDefault(listOf(BackendConfig()))
    }

    private fun decodeUsage(raw: String?): Map<String, TokenUsage> {
        if (raw.isNullOrBlank()) return emptyMap()
        return runCatching {
            json.decodeFromString<Map<String, TokenUsage>>(raw)
        }.getOrDefault(emptyMap())
    }
}
