package me.siwi.irsclaw.logic

import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import me.siwi.irsclaw.IrsClawApplication
import me.siwi.irsclaw.data.api.ApiException
import me.siwi.irsclaw.data.api.ClawApi
import me.siwi.irsclaw.data.model.AgentUpsertRequest
import me.siwi.irsclaw.data.model.AppMessage
import me.siwi.irsclaw.data.model.ChatRequest
import me.siwi.irsclaw.data.model.ClawAgent
import me.siwi.irsclaw.data.model.ClawConfig
import me.siwi.irsclaw.data.model.DonePayload
import me.siwi.irsclaw.data.model.EvaluationPayload
import me.siwi.irsclaw.data.model.ImageGeneratedPayload
import me.siwi.irsclaw.data.model.MessageItem
import me.siwi.irsclaw.data.model.SessionMeta
import me.siwi.irsclaw.data.model.StatsResponse
import me.siwi.irsclaw.data.model.TokenUsage
import me.siwi.irsclaw.data.model.ToolExecutedPayload
import me.siwi.irsclaw.data.model.toMessageItems
import me.siwi.irsclaw.data.settings.AppSettings
import me.siwi.irsclaw.data.settings.Appearance
import me.siwi.irsclaw.data.settings.SettingsStore

enum class ConnectionState {
    DISCONNECTED,
    WAITING_FOR_HEALTH,
    CONNECTED,
    FAILED,
}

data class ClawUiState(
    val connectionState: ConnectionState = ConnectionState.DISCONNECTED,
    val errorMessage: String? = null,
    val sessions: List<SessionMeta> = emptyList(),
    val currentSession: SessionMeta? = null,
    val messages: List<MessageItem> = emptyList(),
    val agents: List<ClawAgent> = emptyList(),
    val currentAgentId: String = "default",
    val config: ClawConfig? = null,
    val tools: List<me.siwi.irsclaw.data.model.ToolInfo> = emptyList(),
    val skills: List<me.siwi.irsclaw.data.model.SkillInfo> = emptyList(),
    val plugins: List<me.siwi.irsclaw.data.model.PluginInfo> = emptyList(),
    val stats: StatsResponse? = null,
    val sessionTokenUsage: Map<String, TokenUsage> = emptyMap(),
    val isProcessing: Boolean = false,
    /** Bumped on every streaming mutation so the list can auto-scroll, mirroring iOS messageVersion. */
    val messageVersion: Int = 0,
)

/**
 * Port of the iOS `ClawService` god-store: connection gate, session management,
 * and the SSE chat loop with 8KB token batching. All state lives in one
 * [MutableStateFlow]; streaming events mutate it from the IO dispatcher
 * (StateFlow is thread-safe and updates stay ordered).
 */
class ClawViewModel(application: IrsClawApplication) : AndroidViewModel(application) {

    private val api: ClawApi = application.container.clawApi
    private val settingsStore: SettingsStore = application.container.settingsStore

    private val _state = MutableStateFlow(ClawUiState())
    val state: StateFlow<ClawUiState> = _state.asStateFlow()

    val settings: StateFlow<AppSettings?> = settingsStore.settings
        .stateIn(viewModelScope, SharingStarted.Eagerly, null)

    private var currentSessionId: String? = null
    private var healthJob: Job? = null
    private var sseJob: Job? = null

    // MARK: - Connection gate (5 attempts, 1s apart, 3s timeout — mirrors iOS)

    fun connectToBackend() {
        if (_state.value.connectionState == ConnectionState.WAITING_FOR_HEALTH) return
        healthJob?.cancel()
        sseJob?.cancel()
        _state.update { it.copy(connectionState = ConnectionState.WAITING_FOR_HEALTH, errorMessage = null) }
        healthJob = viewModelScope.launch {
            for (attempt in 1..5) {
                if (api.health()) {
                    _state.update { it.copy(connectionState = ConnectionState.CONNECTED) }
                    loadInitialData()
                    return@launch
                }
                if (attempt < 5) delay(1_000)
            }
            _state.update {
                it.copy(
                    connectionState = ConnectionState.FAILED,
                    errorMessage = "无法连接到后端，请检查服务地址与网络",
                )
            }
        }
    }

    fun restartBackend() {
        _state.update {
            it.copy(connectionState = ConnectionState.DISCONNECTED, messages = emptyList(), currentSession = null, sessions = emptyList())
        }
        currentSessionId = null
        connectToBackend()
    }

    fun disconnect() {
        healthJob?.cancel()
        sseJob?.cancel()
        _state.update {
            it.copy(
                connectionState = ConnectionState.DISCONNECTED,
                messages = emptyList(),
                sessions = emptyList(),
                currentSession = null,
                agents = emptyList(),
            )
        }
        currentSessionId = null
    }

    private suspend fun loadInitialData() {
        fetchSessions()
        fetchCurrentSession()
        fetchAgents(autoCreateDefault = true)
        fetchConfig()
    }

    // MARK: - Sessions

    fun fetchSessions() {
        viewModelScope.launch {
            try {
                val sessions = api.fetchSessions()
                _state.update { it.copy(sessions = sessions) }
            } catch (e: ApiException) {
                handleAuth(e)
            } catch (_: Exception) {
                // Session list refresh is best-effort, like iOS.
            }
        }
    }

    private suspend fun fetchCurrentSession() {
        try {
            val current = api.fetchCurrentSession()
            currentSessionId = current?.id
            _state.update {
                it.copy(
                    currentSession = it.sessions.firstOrNull { s -> s.id == current?.id },
                    messages = current?.messages?.toMessageItems() ?: emptyList(),
                    currentAgentId = current?.agentId ?: it.currentAgentId,
                    messageVersion = it.messageVersion + 1,
                )
            }
            attachPersistedTokenUsage()
        } catch (_: Exception) {
            // No current session yet — fine on a fresh backend.
        }
    }

    fun createSession() {
        viewModelScope.launch {
            try {
                val newSession = api.createSession(_state.value.currentAgentId)
                fetchSessions()
                switchToSession(newSession.id)
            } catch (e: ApiException) {
                handleAuth(e)
            } catch (_: Exception) {
            }
        }
    }

    fun switchToSession(id: String) {
        sseJob?.cancel()
        sseJob = null
        viewModelScope.launch {
            try {
                api.switchSession(id)
                _state.update {
                    it.copy(
                        currentSession = it.sessions.firstOrNull { s -> s.id == id },
                        currentAgentId = it.sessions.firstOrNull { s -> s.id == id }?.agentId ?: it.currentAgentId,
                    )
                }
                currentSessionId = id
                fetchSessionMessages(id)
            } catch (e: ApiException) {
                handleAuth(e)
            } catch (_: Exception) {
            }
        }
    }

    fun fetchSessionMessages(id: String) {
        viewModelScope.launch {
            try {
                val detail = api.fetchSessionMessages(id)
                _state.update {
                    it.copy(
                        messages = detail.messages.toMessageItems(),
                        currentAgentId = detail.agentId ?: it.currentAgentId,
                        messageVersion = it.messageVersion + 1,
                    )
                }
                attachPersistedTokenUsage()
            } catch (e: ApiException) {
                handleAuth(e)
            } catch (_: Exception) {
            }
        }
    }

    fun deleteSession(id: String) {
        viewModelScope.launch {
            try {
                api.deleteSession(id)
            } catch (_: Exception) {
            }
            fetchSessions()
            if (_state.value.currentSession?.id == id) {
                currentSessionId = null
                _state.update { it.copy(currentSession = null, messages = emptyList()) }
            }
        }
    }

    fun switchAgent(agentId: String) {
        if (agentId == _state.value.currentAgentId) return
        if (_state.value.connectionState != ConnectionState.CONNECTED) return
        _state.update { it.copy(currentAgentId = agentId) }
        val existing = _state.value.sessions.firstOrNull { it.agentId == agentId }
        if (existing != null) switchToSession(existing.id) else createSession()
    }

    // MARK: - Chat (send + SSE stream)

    fun sendMessage(text: String) {
        if (text.isBlank() || _state.value.isProcessing) return
        sseJob?.cancel()

        _state.update {
            it.copy(
                messages = it.messages + MessageItem(message = AppMessage.User(text)),
                isProcessing = true,
                errorMessage = null,
                messageVersion = it.messageVersion + 1,
            )
        }

        var tokenBuffer = ""
        sseJob = viewModelScope.launch {
            try {
                api.chatSse(ChatRequest(message = text, agentId = _state.value.currentAgentId)) { event, data ->
                    if (event == "token") {
                        tokenBuffer += data
                        if (tokenBuffer.encodeToByteArray().size >= TOKEN_FLUSH_BYTES) {
                            appendAssistantText(tokenBuffer)
                            tokenBuffer = ""
                        }
                    } else {
                        if (tokenBuffer.isNotEmpty()) {
                            appendAssistantText(tokenBuffer)
                            tokenBuffer = ""
                        }
                        handleSseEvent(event, data)
                    }
                }
                if (tokenBuffer.isNotEmpty()) appendAssistantText(tokenBuffer)
                _state.update { it.copy(isProcessing = false) }
                fetchSessions()
            } catch (e: CancellationException) {
                throw e
            } catch (e: ApiException) {
                _state.update {
                    it.copy(
                        isProcessing = false,
                        errorMessage = e.message,
                        messages = it.messages + MessageItem(message = AppMessage.Error(e.message ?: "error")),
                    )
                }
            } catch (e: Exception) {
                _state.update {
                    it.copy(
                        isProcessing = false,
                        errorMessage = "Stream error: ${e.message}",
                        messages = it.messages + MessageItem(message = AppMessage.Error(e.message ?: "error")),
                    )
                }
            }
        }
    }

    fun stopStreaming() {
        sseJob?.cancel()
        sseJob = null
        _state.update { it.copy(isProcessing = false) }
    }

    /** Thread-safe: called from the SSE reader on IO and flushed under the state lock. */
    private fun appendAssistantText(text: String) {
        if (text.isEmpty()) return
        _state.update { s ->
            val last = s.messages.lastOrNull()
            val updated = if (last?.message is AppMessage.Assistant) {
                s.messages.dropLast(1) + MessageItem(
                    id = last.id,
                    message = AppMessage.Assistant((last.message as AppMessage.Assistant).text + text),
                    tokenUsage = last.tokenUsage,
                )
            } else {
                s.messages + MessageItem(message = AppMessage.Assistant(text))
            }
            s.copy(messages = updated, messageVersion = s.messageVersion + 1)
        }
    }

    private fun handleSseEvent(event: String, data: String) {
        when (event) {
            "reasoning" -> if (data.isNotEmpty()) {
                _state.update { s ->
                    val last = s.messages.lastOrNull()
                    val updated = if (last?.message is AppMessage.Reasoning) {
                        s.messages.dropLast(1) + MessageItem(
                            id = last.id,
                            message = AppMessage.Reasoning((last.message as AppMessage.Reasoning).text + data),
                        )
                    } else {
                        s.messages + MessageItem(message = AppMessage.Reasoning(data))
                    }
                    s.copy(messages = updated, messageVersion = s.messageVersion + 1)
                }
            }

            "status" -> _state.update {
                it.copy(
                    messages = it.messages + MessageItem(message = AppMessage.Status(data)),
                    messageVersion = it.messageVersion + 1,
                )
            }

            "tool_executed" -> runCatching { json.decodeFromString<ToolExecutedPayload>(data) }.getOrNull()
                ?.let { payload ->
                    _state.update {
                        it.copy(
                            messages = it.messages + MessageItem(
                                message = AppMessage.ToolCall(
                                    name = payload.name,
                                    args = payload.args,
                                    result = payload.result,
                                    step = payload.step,
                                    totalSteps = payload.totalSteps,
                                ),
                            ),
                            messageVersion = it.messageVersion + 1,
                        )
                    }
                }

            "image_generated" -> runCatching { json.decodeFromString<ImageGeneratedPayload>(data) }.getOrNull()
                ?.let { payload ->
                    val filename = payload.path.substringAfterLast('/')
                    val url = "${settings.value?.currentBackend?.url?.trimEnd('/') ?: ""}/api/images/$filename"
                    _state.update {
                        it.copy(
                            messages = it.messages + MessageItem(
                                message = AppMessage.Image(
                                    path = payload.path,
                                    altText = payload.altText,
                                    width = payload.width,
                                    height = payload.height,
                                    format = payload.format,
                                    url = url,
                                ),
                            ),
                            messageVersion = it.messageVersion + 1,
                        )
                    }
                }

            "new_round" -> Unit

            "done" -> {
                val payload = runCatching { json.decodeFromString<DonePayload>(data) }.getOrNull()
                if (payload != null) {
                    payload.sessionId?.let { currentSessionId = it }
                    val usage = payload.usage
                    if (usage != null) {
                        _state.update { s ->
                            val messages = s.messages.toMutableList()
                            val lastIdx = messages.indexOfLast { it.message is AppMessage.Assistant }
                            if (lastIdx >= 0) {
                                val m = messages[lastIdx]
                                messages[lastIdx] = m.copy(tokenUsage = usage)
                            }
                            s.copy(messages = messages, isProcessing = false)
                        }
                        val sid = _state.value.currentSession?.id ?: currentSessionId
                        if (sid != null) {
                            viewModelScope.launch {
                                settingsStore.addSessionUsage(sid, usage)
                                _state.update { s ->
                                    s.copy(sessionTokenUsage = s.sessionTokenUsage + (sid to ((s.sessionTokenUsage[sid] ?: TokenUsage()) + usage)))
                                }
                            }
                        }
                    }
                    payload.quality?.let { quality ->
                        _state.update {
                            it.copy(
                                messages = it.messages + MessageItem(
                                    message = AppMessage.Quality(
                                        score = quality.score,
                                        complete = quality.complete,
                                        issues = quality.issues,
                                        referencesValid = quality.referencesValid ?: false,
                                    ),
                                ),
                                messageVersion = it.messageVersion + 1,
                            )
                        }
                    }
                }
                _state.update { it.copy(isProcessing = false) }
            }

            "evaluation" -> runCatching { json.decodeFromString<EvaluationPayload>(data) }.getOrNull()?.let { payload ->
                _state.update {
                    it.copy(
                        messages = it.messages + MessageItem(
                            message = AppMessage.Evaluation(payload.tool, payload.valid, payload.issues),
                        ),
                        messageVersion = it.messageVersion + 1,
                    )
                }
            }

            // Defensive: older builds emitted quality as a standalone event.
            "quality_score" -> runCatching { json.decodeFromString<me.siwi.irsclaw.data.model.QualityInfo>(data) }.getOrNull()?.let { quality ->
                _state.update {
                    it.copy(
                        messages = it.messages + MessageItem(
                            message = AppMessage.Quality(quality.score, quality.complete, quality.issues, quality.referencesValid ?: false),
                        ),
                        messageVersion = it.messageVersion + 1,
                    )
                }
            }

            "error" -> _state.update {
                it.copy(
                    messages = it.messages + MessageItem(message = AppMessage.Error(data)),
                    messageVersion = it.messageVersion + 1,
                )
            }
        }
    }

    fun postFeedback(sessionId: String, positive: Boolean, message: String? = null) {
        viewModelScope.launch {
            try {
                api.postFeedback(sessionId, positive, message)
            } catch (_: Exception) {
            }
        }
    }

    // MARK: - Agents

    fun fetchAgents(autoCreateDefault: Boolean = false) {
        viewModelScope.launch {
            try {
                var agents = api.fetchAgents().filterNot { it.isSubAgent == true }
                if (autoCreateDefault && agents.isEmpty()) {
                    api.createAgent(AgentUpsertRequest(id = "default"))
                    agents = api.fetchAgents().filterNot { it.isSubAgent == true }
                }
                _state.update {
                    it.copy(
                        agents = agents,
                        currentAgentId = if (agents.none { a -> a.id == it.currentAgentId }) {
                            agents.firstOrNull()?.id ?: it.currentAgentId
                        } else {
                            it.currentAgentId
                        },
                    )
                }
            } catch (e: ApiException) {
                handleAuth(e)
            } catch (_: Exception) {
            }
        }
    }

    fun createAgent(request: AgentUpsertRequest, onDone: (Boolean) -> Unit = {}) {
        viewModelScope.launch {
            val ok = try {
                api.createAgent(request)
                fetchAgents()
                true
            } catch (e: Exception) {
                _state.update { it.copy(errorMessage = e.message) }
                false
            }
            onDone(ok)
        }
    }

    fun updateAgent(id: String, request: AgentUpsertRequest, onDone: (Boolean) -> Unit = {}) {
        viewModelScope.launch {
            val ok = try {
                api.updateAgent(id, request)
                fetchAgents()
                true
            } catch (e: Exception) {
                _state.update { it.copy(errorMessage = e.message) }
                false
            }
            onDone(ok)
        }
    }

    fun deleteAgent(id: String) {
        viewModelScope.launch {
            try {
                api.deleteAgent(id)
            } catch (_: Exception) {
            }
            fetchAgents()
        }
    }

    fun fetchAgentDetail(id: String, onDone: (me.siwi.irsclaw.data.model.AgentDetail?) -> Unit) {
        viewModelScope.launch {
            onDone(try {
                api.fetchAgentDetail(id)
            } catch (_: Exception) {
                null
            })
        }
    }

    // MARK: - Config / Provider

    fun fetchConfig() {
        viewModelScope.launch {
            try {
                val config = api.fetchConfig()
                _state.update { it.copy(config = config) }
            } catch (e: ApiException) {
                handleAuth(e)
            } catch (_: Exception) {
            }
        }
    }

    fun updateLlmConfig(provider: String, apiKey: String, baseUrl: String, model: String? = null) {
        viewModelScope.launch {
            try {
                api.patchLlmConfig(me.siwi.irsclaw.data.model.LlmConfigPatch(provider, apiKey, baseUrl, model))
                settingsStore.setLlmConfig(provider, apiKey, baseUrl)
                fetchConfig()
            } catch (e: Exception) {
                _state.update { it.copy(errorMessage = e.message) }
            }
        }
    }

    // MARK: - Tools / Skills / Plugins / Stats

    fun fetchTools() {
        viewModelScope.launch {
            try {
                _state.update { it.copy(tools = api.fetchTools()) }
            } catch (_: Exception) {
            }
        }
    }

    fun fetchSkills() {
        viewModelScope.launch {
            try {
                _state.update { it.copy(skills = api.fetchSkills()) }
            } catch (_: Exception) {
            }
        }
    }

    fun fetchPlugins() {
        viewModelScope.launch {
            try {
                _state.update { it.copy(plugins = api.fetchPlugins()) }
            } catch (_: Exception) {
            }
        }
    }

    fun fetchStats(period: String) {
        viewModelScope.launch {
            try {
                _state.update { it.copy(stats = api.fetchStats(period)) }
            } catch (_: Exception) {
            }
        }
    }

    fun setAppearance(appearance: Appearance) {
        viewModelScope.launch { settingsStore.setAppearance(appearance) }
    }

    fun clearError() {
        _state.update { it.copy(errorMessage = null) }
    }

    private fun handleAuth(e: ApiException) {
        if (e.code == 401) {
            _state.update { it.copy(errorMessage = e.message) }
        }
    }

    private fun attachPersistedTokenUsage() {
        val sid = currentSessionId ?: return
        val persisted = settings.value?.sessionTokenUsage?.get(sid) ?: return
        _state.update { s ->
            val messages = s.messages.mapIndexed { idx, m ->
                if (idx == s.messages.lastIndex && m.message is AppMessage.Assistant && m.tokenUsage == null) {
                    m.copy(tokenUsage = persisted)
                } else {
                    m
                }
            }
            s.copy(messages = messages, sessionTokenUsage = s.sessionTokenUsage + (sid to persisted))
        }
    }

    companion object {
        private const val TOKEN_FLUSH_BYTES = 8192

        private val json = kotlinx.serialization.json.Json { ignoreUnknownKeys = true }
    }
}
