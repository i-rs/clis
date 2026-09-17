package me.siwi.irsclaw.data.api

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.withContext
import kotlinx.serialization.json.Json
import me.siwi.irsclaw.data.model.AgentDetail
import me.siwi.irsclaw.data.model.AgentUpsertRequest
import me.siwi.irsclaw.data.model.ApiResponse
import me.siwi.irsclaw.data.model.ChatRequest
import me.siwi.irsclaw.data.model.ClawAgent
import me.siwi.irsclaw.data.model.ClawConfig
import me.siwi.irsclaw.data.model.CurrentSession
import me.siwi.irsclaw.data.model.FeedbackRequest
import me.siwi.irsclaw.data.model.LlmConfigPatch
import me.siwi.irsclaw.data.model.PluginInfo
import me.siwi.irsclaw.data.model.SessionDetail
import me.siwi.irsclaw.data.model.SessionMeta
import me.siwi.irsclaw.data.model.SkillInfo
import me.siwi.irsclaw.data.model.StatsResponse
import me.siwi.irsclaw.data.model.ToolInfo
import me.siwi.irsclaw.data.settings.SettingsStore
import okhttp3.MediaType.Companion.toMediaType
import okhttp3.OkHttpClient
import okhttp3.Request
import okhttp3.RequestBody.Companion.toRequestBody
import java.io.IOException
import java.util.concurrent.TimeUnit
import kotlin.coroutines.coroutineContext

/** Error surfaced from a failed API exchange; [code] is 0 when no HTTP status applies. */
class ApiException(message: String, val code: Int = 0) : Exception(message) {
    companion object {
        const val AUTH_FAILED = "认证失败，请在设置中检查 Auth Token"
    }
}

/**
 * REST + SSE client for `claw serve`, mirroring the iOS `ClawService` networking:
 * `{success, data, error}` envelopes, snake_case JSON, and timeouts of
 * 3s (health) / 10s (GET·DELETE·PUT) / 60s (POST) / 300s (SSE chat).
 */
class ClawApi(
    private val settings: SettingsStore,
    baseClient: OkHttpClient,
) {
    private val json = Json {
        ignoreUnknownKeys = true
        explicitNulls = false
        encodeDefaults = false
    }
    private val jsonBody = "application/json; charset=utf-8".toMediaType()

    private val client = baseClient.newBuilder()
        .connectTimeout(10, TimeUnit.SECONDS)
        .build()

    // MARK: - Health

    /** @return true when /api/health answers 200 within 3 seconds. */
    suspend fun health(): Boolean {
        val backend = backend()
        val request = baseRequest(backend)
            .url("${backend.url}/api/health")
            .get()
            .build()
        val call = client.newBuilder()
            .callTimeout(3, TimeUnit.SECONDS)
            .build()
            .newCall(request)
        return withContext(Dispatchers.IO) {
            runCatching { call.execute().use { it.isSuccessful } }.getOrDefault(false)
        }
    }

    // MARK: - Chat (POST → SSE stream)

    /**
     * POSTs [request] to /api/chat and consumes the SSE response, invoking
     * [onEvent] for every parsed event. [onEvent] runs on the IO dispatcher;
     * handlers must only touch thread-safe state (StateFlow updates).
     * Cancelling the caller's coroutine aborts the underlying HTTP call.
     */
    suspend fun chatSse(request: ChatRequest, onEvent: suspend (event: String, data: String) -> Unit) {
        val backend = backend()
        val httpRequest = baseRequest(backend)
            .url("${backend.url}/api/chat")
            .post(json.encodeToString(request).toRequestBody(jsonBody))
            .build()
        val sseClient = client.newBuilder()
            .readTimeout(300, TimeUnit.SECONDS)
            .writeTimeout(60, TimeUnit.SECONDS)
            .callTimeout(0, TimeUnit.SECONDS)
            .build()
        val call = sseClient.newCall(httpRequest)
        val job: Job? = coroutineContext[Job]
        val handle = job?.invokeOnCompletion { call.cancel() }
        try {
            withContext(Dispatchers.IO) {
                call.execute().use { resp ->
                    when {
                        resp.code == 401 -> throw ApiException(ApiException.AUTH_FAILED, 401)
                        resp.code == 429 -> throw ApiException("并发请求已达上限，请稍后再试", 429)
                        !resp.isSuccessful -> throw ApiException("Server error: ${resp.code}", resp.code)
                    }
                    val source = resp.body?.source() ?: throw ApiException("Empty response body", resp.code)
                    val parser = SseParser()
                    while (true) {
                        val line = source.readUtf8Line() ?: break
                        val event = parser.feed(line) ?: continue
                        onEvent(event.event, event.data)
                    }
                }
            }
        } catch (e: IOException) {
            if (job?.isCancelled == true) {
                throw CancellationException("stream cancelled").initCause(e)
            }
            throw ApiException("Stream error: ${e.message ?: "network failure"}")
        } finally {
            handle?.dispose()
        }
    }

    // MARK: - Sessions

    suspend fun fetchSessions(): List<SessionMeta> =
        exchange("/api/sessions", "GET") ?: emptyList()

    /** Null when the backend has no current session yet. */
    suspend fun fetchCurrentSession(): CurrentSession? =
        exchange("/api/sessions/current", "GET")

    suspend fun fetchSessionMessages(id: String): SessionDetail =
        exchange("/api/sessions/$id", "GET") ?: throw ApiException("Empty response data")

    suspend fun createSession(agentId: String): SessionMeta =
        exchange<SessionMeta>(
            "/api/sessions",
            "POST",
            bodyJson = """{"agent_id": ${json.encodeToString(agentId)}}""",
        ) ?: throw ApiException("Empty response data")

    suspend fun switchSession(id: String): SessionMeta? =
        exchange("/api/sessions/$id/switch", "POST", bodyJson = "{}")

    suspend fun deleteSession(id: String) {
        exchange<Unit>("/api/sessions/$id", "DELETE")
    }

    suspend fun postFeedback(sessionId: String, positive: Boolean, message: String? = null) {
        exchange<Unit>("/api/sessions/$sessionId/feedback", "POST", json.encodeToString(FeedbackRequest(positive, message)))
    }

    // MARK: - Agents

    suspend fun fetchAgents(): List<ClawAgent> = exchange("/api/agents", "GET") ?: emptyList()

    suspend fun fetchAgentDetail(id: String): AgentDetail =
        exchange("/api/agents/$id", "GET") ?: throw ApiException("Empty response data")

    suspend fun createAgent(request: AgentUpsertRequest) {
        exchange<Map<String, String>>("/api/agents", "POST", json.encodeToString(request))
    }

    suspend fun updateAgent(id: String, request: AgentUpsertRequest) {
        exchange<Map<String, String>>("/api/agents/$id", "PUT", json.encodeToString(request))
    }

    suspend fun deleteAgent(id: String) {
        exchange<Unit>("/api/agents/$id", "DELETE")
    }

    // MARK: - Config / Provider

    suspend fun fetchConfig(): ClawConfig = exchange("/api/config", "GET") ?: ClawConfig()

    suspend fun patchLlmConfig(patch: LlmConfigPatch) {
        exchange<Unit>("/api/config", "PATCH", json.encodeToString(patch))
    }

    // MARK: - Tools / Skills / Plugins / Stats

    suspend fun fetchTools(): List<ToolInfo> = exchange("/api/tools", "GET") ?: emptyList()

    suspend fun fetchSkills(): List<SkillInfo> = exchange("/api/skills", "GET") ?: emptyList()

    suspend fun fetchPlugins(): List<PluginInfo> = exchange("/api/plugins", "GET") ?: emptyList()

    suspend fun fetchStats(period: String): StatsResponse = exchange("/api/stats?period=$period", "GET") ?: StatsResponse()

    // MARK: - HTTP plumbing

    private suspend fun backend() = settings.currentBackend().let { it.copy(url = it.url.trimEnd('/')) }

    private fun baseRequest(backend: me.siwi.irsclaw.data.settings.BackendConfig): Request.Builder {
        val builder = Request.Builder()
        if (backend.authToken.isNotEmpty()) {
            builder.header("Authorization", "Bearer ${backend.authToken}")
        }
        return builder
    }

    /**
     * Performs the request and unwraps the `{success, data, error}` envelope.
     * Returns the envelope's `data`, which is null for Unit-style endpoints.
     */
    private suspend inline fun <reified T> exchange(
        path: String,
        method: String,
        bodyJson: String? = null,
    ): T? {
        val backend = backend()
        val timeout = if (method == "POST" || method == "PATCH") 60L else 10L
        val body = when {
            bodyJson != null -> bodyJson.toRequestBody(jsonBody)
            method == "POST" || method == "PUT" || method == "PATCH" -> "{}".toRequestBody(jsonBody)
            else -> null
        }
        val request = baseRequest(backend)
            .url("${backend.url}$path")
            .method(method, body)
            .build()
        val call = client.newBuilder()
            .callTimeout(timeout, TimeUnit.SECONDS)
            .build()
            .newCall(request)
        return withContext(Dispatchers.IO) {
            try {
                call.execute().use { resp ->
                    val raw = resp.body?.string() ?: ""
                    if (resp.code == 401) throw ApiException(ApiException.AUTH_FAILED, 401)
                    if (!resp.isSuccessful) {
                        val detail = runCatching {
                            json.decodeFromString<ApiResponse<Unit>>(raw).error
                        }.getOrNull()
                        throw ApiException(detail ?: "Server error: ${resp.code}", resp.code)
                    }
                    val envelope = runCatching { json.decodeFromString<ApiResponse<T>>(raw) }
                        .getOrElse { throw ApiException("Malformed response: ${it.message}") }
                    if (!envelope.success) throw ApiException(envelope.error ?: "Unknown error")
                    envelope.data
                }
            } catch (e: IOException) {
                throw ApiException("Network error: ${e.message ?: "connection failed"}")
            }
        }
    }
}
