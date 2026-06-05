import Foundation
import Combine

// MARK: - Connection State

/// Describes the current state of the backend connection.
enum ConnectionState: Equatable {
    case disconnected
    case waitingForHealth
    case connected
    case failed(String)

    var isConnected: Bool { self == .connected }
    var isConnecting: Bool {
        switch self {
        case .waitingForHealth: return true
        default: return false
        }
    }
    var errorMessage: String? {
        if case .failed(let msg) = self { return msg }
        return nil
    }
}

// MARK: - ClawService

/// Manages the i-rs-claw backend connection and provides a REST API client.
@MainActor
class ClawService: ObservableObject {
    // MARK: - Published State

    @Published var connectionState: ConnectionState = .disconnected
    @Published var isProcessing = false
    @Published var sessions: [ClawSession] = []
    @Published var currentSession: ClawSession?
    @Published var messages: [MessageItem] = []
    @Published var agents: [ClawAgent] = []
    @Published var config: ClawConfig?
    @Published var tools: [ToolInfo] = []
    @Published var skills: [SkillInfo] = []
    @Published var plugins: [PluginInfo] = []
    @Published var errorMessage: String?
    @Published var backendPid: Int32?
    /// Current agent ID for the active chat session.
    @Published var currentAgentId: String = "default"
    /// Incremented on each message update to trigger scroll in ChatView
    @Published var messageVersion = 0
    /// Token usage per session, persisted across session switches and app restarts.
    @Published var sessionTokenUsage: [String: TokenUsage] = [:]
    @Published var stats: StatsResponse?
    @Published var isLoadingStats = false

    /// Total token usage aggregated across all sessions.
    var totalTokenUsage: TokenUsage {
        sessionTokenUsage.values.reduce(TokenUsage(promptTokens: 0, completionTokens: 0, estimatedCostUsd: 0)) { acc, usage in
            acc + usage
        }
    }

    // MARK: - Private Properties

    private static let defaultBaseURL = "http://127.0.0.1:3000"
    private let decoder = JSONDecoder()
    private var sseTask: Task<Void, Never>?
    private var healthCheckTask: Task<Void, Never>?

    @Published var backendConfigs: [BackendConfig] = []
    @Published var currentBackendId: UUID?

    init() {
        loadBackendConfigs()
        loadSessionTokenUsage()
    }

    /// The server URL used for all API requests, from active backend config.
    private var baseURL: String {
        activeConfig?.url ?? UserDefaults.standard.string(forKey: "server_url") ?? Self.defaultBaseURL
    }

    /// Auth token for Bearer authentication, from active backend config.
    private var authToken: String {
        activeConfig?.authToken ?? UserDefaults.standard.string(forKey: "claw_auth_token") ?? ""
    }

    private var activeConfig: BackendConfig? {
        guard let id = currentBackendId else { return nil }
        return backendConfigs.first(where: { $0.id == id })
    }

    // MARK: - Backend Config Management

    private static let configsKey = "backend_configs"
    private static let activeKey = "current_backend_id"

    func loadBackendConfigs() {
        // Migrate old single-server config if exists
        let hasOldURL = UserDefaults.standard.string(forKey: "server_url") != nil
        let hasOldToken = UserDefaults.standard.string(forKey: "claw_auth_token") != nil
        let hasNewConfigs = UserDefaults.standard.data(forKey: Self.configsKey) != nil

        if !hasNewConfigs, hasOldURL || hasOldToken {
            let oldURL = UserDefaults.standard.string(forKey: "server_url") ?? Self.defaultBaseURL
            let oldToken = UserDefaults.standard.string(forKey: "claw_auth_token") ?? ""
            let migrated = BackendConfig(name: "Default", url: oldURL, authToken: oldToken)
            backendConfigs = [migrated]
            currentBackendId = migrated.id
            saveBackendConfigs()
            // Clean up old keys
            UserDefaults.standard.removeObject(forKey: "server_url")
            UserDefaults.standard.removeObject(forKey: "claw_auth_token")
            return
        }

        // Load from new storage
        if let data = UserDefaults.standard.data(forKey: Self.configsKey),
           let configs = try? JSONDecoder().decode([BackendConfig].self, from: data) {
            backendConfigs = configs
        } else {
            // No configs at all — create default
            let default_ = BackendConfig(name: "Default", url: Self.defaultBaseURL)
            backendConfigs = [default_]
            currentBackendId = default_.id
            saveBackendConfigs()
            return
        }

        if let idData = UserDefaults.standard.data(forKey: Self.activeKey),
           let id = try? JSONDecoder().decode(UUID.self, from: idData),
           backendConfigs.contains(where: { $0.id == id }) {
            currentBackendId = id
        } else {
            currentBackendId = backendConfigs.first?.id
        }
    }

    private func saveBackendConfigs() {
        if let data = try? JSONEncoder().encode(backendConfigs) {
            UserDefaults.standard.set(data, forKey: Self.configsKey)
        }
        if let id = currentBackendId, let data = try? JSONEncoder().encode(id) {
            UserDefaults.standard.set(data, forKey: Self.activeKey)
        }
    }

    func addBackend(name: String, url: String, authToken: String = "") {
        let config = BackendConfig(name: name, url: url, authToken: authToken)
        backendConfigs.append(config)
        currentBackendId = config.id
        saveBackendConfigs()
        restartBackend()
    }

    func updateBackend(_ config: BackendConfig) {
        guard let idx = backendConfigs.firstIndex(where: { $0.id == config.id }) else { return }
        backendConfigs[idx] = config
        saveBackendConfigs()
        restartBackend()
    }

    func deleteBackend(_ id: UUID) {
        backendConfigs.removeAll(where: { $0.id == id })
        if currentBackendId == id {
            currentBackendId = backendConfigs.first?.id
        }
        saveBackendConfigs()
        restartBackend()
    }

    func switchBackend(to id: UUID) {
        guard backendConfigs.contains(where: { $0.id == id }), id != currentBackendId else { return }
        currentBackendId = id
        saveBackendConfigs()
        restartBackend()
    }

    /// Update the auth token of the active backend and reconnect.
    func updateAuthToken(_ newToken: String) {
        guard var config = activeConfig else {
            // Fallback to old key-based storage if no configs exist
            UserDefaults.standard.set(newToken, forKey: "claw_auth_token")
            print("[ClawService] Auth token updated (legacy)")
            restartBackend()
            return
        }
        config.authToken = newToken
        updateBackend(config)
    }

    // MARK: - Session Token Usage Persistence

    private static let tokenUsageKey = "session_token_usage_v2"

    private func saveSessionTokenUsage() {
        guard let data = try? JSONEncoder().encode(sessionTokenUsage) else { return }
        UserDefaults.standard.set(data, forKey: Self.tokenUsageKey)
    }

    private func loadSessionTokenUsage() {
        guard let data = UserDefaults.standard.data(forKey: Self.tokenUsageKey),
              let dict = try? JSONDecoder().decode([String: TokenUsage].self, from: data)
        else { return }
        sessionTokenUsage = dict
    }

    private func attachPersistedTokenUsage(for sessionId: String) {
        // sessionTokenUsage now stores cumulative per-session totals for the global panel.
        // Per-message token usage is only available during live SSE streaming.
    }

    /// Clear the auth token of the active backend.
    func clearAuthToken() {
        guard var config = activeConfig else {
            UserDefaults.standard.removeObject(forKey: "claw_auth_token")
            restartBackend()
            return
        }
        config.authToken = ""
        updateBackend(config)
    }

    /// Add Bearer auth header if token is set.
    private func addAuthHeader(_ request: inout URLRequest) {
        if !authToken.isEmpty {
            request.setValue("Bearer \(authToken)", forHTTPHeaderField: "Authorization")
        }
    }

    /// Update the active backend URL or create one if no config exists.
    func updateServerURL(_ newURL: String) {
        let url = newURL.trimmingCharacters(in: .whitespaces)
        guard !url.isEmpty else { return }
        if var config = activeConfig {
            config.url = url
            updateBackend(config)
        } else {
            addBackend(name: "Server", url: url)
        }
    }

    /// Reset active backend URL to default.
    func resetServerURL() {
        guard var config = activeConfig else {
            UserDefaults.standard.removeObject(forKey: "server_url")
            restartBackend()
            return
        }
        config.url = Self.defaultBaseURL
        updateBackend(config)
    }

    var serverURLDisplay: String {
        baseURL
    }

    // MARK: - Backend Connection

    /// Connect to an already-running i-rs-claw dashboard backend.
    /// Polls the health endpoint for up to 5 seconds, then loads initial data.
    func connectToBackend() {
        guard !connectionState.isConnecting && !connectionState.isConnected else { return }
        connectionState = .waitingForHealth
        print("[ClawService] connectToBackend: starting health check...")

        Task { [weak self] in
            var attempts = 0
            while attempts < 5 {
                if Task.isCancelled { return }
                if await self?.checkHealth() == true {
                    print("[ClawService] health check SUCCEEDED")
                    guard let self else { return }
                    self.connectionState = .connected
                    self.errorMessage = nil
                    // Load initial data
                    await self.loadInitialData()
                    print("[ClawService] initial data loaded")
                    return
                }
                print("[ClawService] health check attempt \(attempts + 1) failed")
                attempts += 1
                try? await Task.sleep(nanoseconds: 1_000_000_000)
            }
            print("[ClawService] health check: all attempts failed, going to disconnected")
            self?.connectionState = .disconnected
        }
    }

    /// Disconnect from the backend and reset state.
    func stopBackend() {
        healthCheckTask?.cancel()
        healthCheckTask = nil
        sseTask?.cancel()
        sseTask = nil
        connectionState = .disconnected
        backendPid = nil
        sessions = []
        messages = []
        currentSession = nil
    }

    /// Reconnect to the backend.
    func restartBackend() {
        stopBackend()
        connectToBackend()
    }

    func updateLLMConfig(provider: String, apiKey: String, baseURL: String, model: String? = nil) async {
        guard let url = URL(string: "\(self.baseURL)/api/config") else { return }
        var request = URLRequest(url: url)
        request.httpMethod = "PATCH"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        var body: [String: Any] = ["provider": provider, "api_key": apiKey, "base_url": baseURL]
        if let model { body["model"] = model }
        request.httpBody = try? JSONSerialization.data(withJSONObject: body)
        do {
            let (data, resp) = try await URLSession.shared.data(for: request)
            if let http = resp as? HTTPURLResponse, http.statusCode == 200 {
                print("[updateLLMConfig] saved provider=\(provider) base_url=\(baseURL)")
            } else {
                print("[updateLLMConfig] failed: status=\((resp as? HTTPURLResponse)?.statusCode ?? 0)")
            }
            _ = data
        } catch {
            print("[updateLLMConfig] error: \(error.localizedDescription)")
        }
    }

    // MARK: - Health Check

    /// Check backend health by calling /api/health.
    private func checkHealth() async -> Bool {
        var request = URLRequest(url: URL(string: "\(baseURL)/api/health")!)
        request.timeoutInterval = 3
        do {
            let (_, response) = try await URLSession.shared.data(for: request)
            let ok = (response as? HTTPURLResponse)?.statusCode == 200
            print("[checkHealth] GET /api/health -> \(ok ? "OK" : "non-200")")
            return ok
        } catch {
            print("[checkHealth] error: \(error.localizedDescription)")
            return false
        }
    }

    /// Load essential data for the chat UI.
    private func loadInitialData() async {
        print("[loadInitialData] fetching sessions...")
        await fetchSessions()
        print("[loadInitialData] sessions loaded: \(sessions.count)")

        print("[loadInitialData] fetching current session...")
        await fetchCurrentSession()
        print("[loadInitialData] current session: \(currentSession?.id ?? "nil")")

        print("[loadInitialData] fetching agents...")
        await withTimeout(seconds: 5) { [weak self] in
            await self?.fetchAgents()
        }
        print("[loadInitialData] agents loaded: \(agents.count)")

        if agents.isEmpty {
            print("[loadInitialData] no agents found, creating default...")
            await createAgent(id: "default", provider: nil, model: nil, apiKey: nil, baseURL: nil, systemPrompt: nil, providerRef: nil)
            await fetchAgents()
        }

        if !agents.contains(where: { $0.id == currentAgentId }) {
            currentAgentId = "default"
        }

        print("[loadInitialData] fetching config...")
        await withTimeout(seconds: 5) { [weak self] in
            await self?.fetchConfig()
        }
        print("[loadInitialData] config loaded")
    }

    /// Run an async operation with a timeout.
    private func withTimeout(seconds: UInt64, operation: @escaping () async -> Void) async {
        let task = Task {
            await operation()
        }
        Task {
            try? await Task.sleep(nanoseconds: seconds * 1_000_000_000)
            task.cancel()
        }
        await task.value
    }

    // MARK: - Session Management

    /// Fetch all sessions from the backend.
    func fetchSessions() async {
        guard let data = await get("/api/sessions") else { return }
        guard let response: ApiResponse<[ClawSession]> = decode(data) else { return }
        if response.success, let sessions = response.data {
            self.sessions = sessions
        }
    }

    /// Fetch the current active session.
    func fetchCurrentSession() async {
        guard let data = await get("/api/sessions/current") else { return }
        struct CurrentSession: Codable {
            let id: String?
            let title: String?
            let messageCount: Int
            let messages: [ClawMessage]
            let agentId: String?

            enum CodingKeys: String, CodingKey {
                case id, title, messages
                case messageCount = "message_count"
                case agentId = "agent_id"
            }
        }
        guard let response: ApiResponse<CurrentSession> = decode(data) else { return }
        if response.success, let cs = response.data {
            if let sid = cs.id {
                self.currentSession = sessions.first(where: { $0.id == sid })
                self.currentSessionId = sid
                attachPersistedTokenUsage(for: sid)
            }
            if let agentId = cs.agentId {
                self.currentAgentId = agentId
            }
            self.messages = cs.messages.flatMap { convertToAppMessages($0) }
            self.messageVersion += 1
        }
    }

    /// Create a new session.
    func createSession() async {
        let body: [String: String] = ["agent_id": currentAgentId]
        guard let bodyData = try? JSONSerialization.data(withJSONObject: body) else { return }
        guard let data = await post("/api/sessions", body: bodyData) else { return }
        guard let response: ApiResponse<ClawSession> = decode(data) else { return }
        if response.success {
            await fetchSessions()
            if let newSession = response.data {
                switchToSession(newSession.id)
            }
        }
    }

    /// Switch to a specific session.
    func switchToSession(_ id: String) {
        sseTask?.cancel()
        sseTask = nil

        Task {
            guard let data = await post("/api/sessions/\(id)/switch") else { return }
            guard let response: ApiResponse<ClawSession> = decode(data) else { return }
            if response.success {
                self.currentSession = sessions.first(where: { $0.id == id })
                self.currentSessionId = id
                await fetchSessionMessages(id)
            }
        }
    }

    /// Delete a session.
    func deleteSession(_ id: String) {
        Task {
            let _ = await delete("/api/sessions/\(id)")
            await fetchSessions()
            if self.currentSession?.id == id {
                self.messages.removeAll()
                self.currentSession = nil
                self.currentSessionId = nil
            }
        }
    }

    /// Load messages for a specific session.
    func fetchSessionMessages(_ id: String) async {
        guard let data = await get("/api/sessions/\(id)") else { return }
        guard let response: ApiResponse<SessionDetail> = decode(data) else { return }
        if response.success, let detail = response.data {
            self.messages = detail.messages.flatMap { convertToAppMessages($0) }
            self.messageVersion += 1
        }
        attachPersistedTokenUsage(for: id)
    }

    func switchAgent(_ agentId: String) async {
        guard agentId != currentAgentId, connectionState.isConnected else { return }
        currentAgentId = agentId

        if let existingSession = sessions.first(where: { $0.agentId == agentId }) {
            switchToSession(existingSession.id)
        } else {
            await createSession()
        }
    }

    // MARK: - Chat (Send Message + SSE Stream)

    /// Send a message and start streaming the response via SSE.
    func sendMessage(_ text: String) {
        guard !text.isEmpty, !isProcessing else { return }

        sseTask?.cancel()

        // Add user message to UI immediately
        messages.append(MessageItem(message: .user(text: text)))
        isProcessing = true
        errorMessage = nil

        Task {
            // POST message
            let body: [String: String] = ["message": text, "agent_id": currentAgentId]
            guard let bodyData = try? JSONSerialization.data(withJSONObject: body) else {
                self.isProcessing = false
                return
            }

            guard let data = await post("/api/chat", body: bodyData) else {
                self.isProcessing = false
                return
            }

            guard let response: ApiResponse<ChatResponse> = decode(data) else {
                self.isProcessing = false
                return
            }

            guard response.success, let chatResponse = response.data else {
                self.isProcessing = false
                if let err = response.error {
                    self.errorMessage = err
                    self.messages.append(MessageItem(message: .error(text: err)))
                }
                return
            }

            let sessionId = chatResponse.sessionId

            // Update current session
            self.currentSessionId = sessionId

            // Start SSE stream
            await streamChat(sessionId: sessionId)

            // Refresh sessions
            await fetchSessions()
        }
    }

    /// Stream chat response via SSE.
    /// Uses byte-level line parsing (not AsyncLineSequence) for reliable real-time streaming.
    /// Batches tokens to reduce SwiftUI rendering pressure.
    private func streamChat(sessionId: String) async {
        let url = URL(string: "\(baseURL)/api/chat/stream/\(sessionId)")!
        var request = URLRequest(url: url)
        request.timeoutInterval = 300
        addAuthHeader(&request)

        sseTask = Task { [weak self] in
            defer {
                Task { @MainActor [weak self] in
                    self?.isProcessing = false
                }
            }

            do {
                let (bytes, response) = try await URLSession.shared.bytes(for: request)
                guard let httpResponse = response as? HTTPURLResponse,
                      httpResponse.statusCode == 200 else {
                    await MainActor.run { [weak self] in
                        self?.messages.append(MessageItem(message: .error(text: "Failed to connect to stream")))
                        self?.isProcessing = false
                    }
                    return
                }

                var lineBuffer = Data()
                var currentEvent = ""
                var currentData = ""
                var tokenBuffer = ""

                for try await byte in bytes {
                    if Task.isCancelled { break }

                    if byte == UInt8(ascii: "\n") {
                        guard let line = String(data: lineBuffer, encoding: .utf8) else {
                            lineBuffer = Data()
                            continue
                        }
                        lineBuffer = Data()

                        if line.hasPrefix("event:") {
                            currentEvent = String(line.dropFirst(6)).trimmingCharacters(in: .whitespaces)
                        } else if line.hasPrefix("data:") {
                            let raw = line.dropFirst(5)
                            let chunk = raw.hasPrefix(" ") ? String(raw.dropFirst()) : String(raw)
                            if currentData.isEmpty {
                                currentData = chunk
                            } else {
                                currentData += "\n" + chunk
                            }
                        } else if line.isEmpty {
                            // End of event — dispatch
                            if !currentEvent.isEmpty {
                                self?.dispatchSseEvent(
                                    event: currentEvent, data: currentData,
                                    tokenBuffer: &tokenBuffer
                                )
                            }
                            currentEvent = ""
                            currentData = ""
                        }
                    } else {
                        lineBuffer.append(byte)
                    }
                }

                // Flush any remaining tokens
                await MainActor.run { [weak self] in
                    guard let self else { return }
                    if !tokenBuffer.isEmpty {
                        appendAssistantText(tokenBuffer)
                        messageVersion += 1
                    }
                }
            } catch {
                if !(error is CancellationError) {
                    await MainActor.run { [weak self] in
                        self?.errorMessage = "Stream error: \(error.localizedDescription)"
                    }
                }
            }
        }
    }

    /// Dispatch an SSE event, batching tokens and flushing on non-token events.
    @MainActor
    private func dispatchSseEvent(event: String, data: String, tokenBuffer: inout String) {
        if event == "token" {
            tokenBuffer += data
            // Flush tokens every 8KB to keep UI responsive during long responses
            if tokenBuffer.utf8.count >= 8192 {
                flushTokenBuffer(&tokenBuffer)
            }
        } else {
            // Non-token event: flush accumulated tokens first
            flushTokenBuffer(&tokenBuffer)
            handleSseEvent(event, data: data)
        }
    }

    /// Flush accumulated token buffer into the last assistant message.
    @MainActor
    private func flushTokenBuffer(_ buffer: inout String) {
        guard !buffer.isEmpty else { return }
        appendAssistantText(buffer)
        buffer = ""
        messageVersion += 1
    }

    /// Append text to the last assistant message (batched token append).
    @MainActor
    private func appendAssistantText(_ text: String) {
        guard !text.isEmpty else { return }
        if let last = messages.last, case .assistant(let existing) = last.message {
            messages[messages.count - 1] = MessageItem(id: last.id, message: .assistant(text: existing + text), tokenUsage: last.tokenUsage)
        } else {
            messages.append(MessageItem(message: .assistant(text: text)))
        }
    }

    /// Handle an SSE event on the main actor (all non-token events).
    @MainActor
    private func handleSseEvent(_ event: String, data: String) {
        switch event {
        case "reasoning":
            if !data.isEmpty {
                if let last = messages.last, case .reasoning(let existing) = last.message {
                    messages[messages.count - 1] = MessageItem(id: last.id, message: .reasoning(text: existing + data))
                } else {
                    messages.append(MessageItem(message: .reasoning(text: data)))
                }
                messageVersion += 1
            }

        case "status":
            messages.append(MessageItem(message: .status(text: data)))
            messageVersion += 1

        case "tool_executed":
            if let json = try? JSONSerialization.jsonObject(with: Data(data.utf8)) as? [String: Any] {
                let name = json["name"] as? String ?? ""
                let args = json["args"] as? String ?? ""
                let result = json["result"] as? String ?? ""
                messages.append(MessageItem(message: .toolCall(name: name, args: args, result: result)))
                messageVersion += 1
            }

        case "new_round":
            break

        case "done":
            isProcessing = false
            // Parse token usage from done event: {"usage": {"prompt_tokens": X, "completion_tokens": Y}}
            if let json = try? JSONSerialization.jsonObject(with: Data(data.utf8)) as? [String: Any],
               let usageDict = json["usage"] as? [String: Any],
               let usageData = try? JSONSerialization.data(withJSONObject: usageDict),
               let usage = try? decoder.decode(TokenUsage.self, from: usageData) {
                // Attach usage to the last assistant message
                if let last = messages.last, case .assistant = last.message {
                    messages[messages.count - 1] = MessageItem(id: last.id, message: last.message, tokenUsage: usage)
                }
                // Accumulate per-session for global tracking
                if let sid = currentSession?.id {
                    let existing = sessionTokenUsage[sid] ?? TokenUsage(promptTokens: 0, completionTokens: 0, estimatedCostUsd: 0)
                    sessionTokenUsage[sid] = existing + usage
                    saveSessionTokenUsage()
                }
            }
            // Parse quality score from done event: {"quality": {"score": "good", "complete": true, ...}}
            if let json = try? JSONSerialization.jsonObject(with: Data(data.utf8)) as? [String: Any],
               let qualityDict = json["quality"] as? [String: Any] {
                let score = String(describing: qualityDict["score"] ?? "")
                let complete = qualityDict["complete"] as? Bool ?? false
                let issues = qualityDict["issues"] as? [String] ?? []
                let referencesValid = qualityDict["references_valid"] as? Bool ?? false
                messages.append(MessageItem(message: .quality(score: score, complete: complete, issues: issues, referencesValid: referencesValid)))
                messageVersion += 1
            }

        case "evaluation":
            if let json = try? JSONSerialization.jsonObject(with: Data(data.utf8)) as? [String: Any] {
                let tool = json["tool"] as? String ?? ""
                let valid = json["valid"] as? Bool ?? false
                let issues = json["issues"] as? [String] ?? []
                messages.append(MessageItem(message: .evaluation(tool: tool, valid: valid, issues: issues)))
                messageVersion += 1
            }

        case "quality_score":
            if let json = try? JSONSerialization.jsonObject(with: Data(data.utf8)) as? [String: Any] {
                let score = String(describing: json["score"] ?? "")
                let complete = json["complete"] as? Bool ?? false
                let issues = json["issues"] as? [String] ?? []
                let referencesValid = json["references_valid"] as? Bool ?? false
                messages.append(MessageItem(message: .quality(score: score, complete: complete, issues: issues, referencesValid: referencesValid)))
                messageVersion += 1
            }

        case "error":
            messages.append(MessageItem(message: .error(text: data)))
            messageVersion += 1

        default:
            break
        }
    }

    // MARK: - Agent Management

    /// Fetch all agents from the backend.
    func fetchAgents() async {
        guard let data = await get("/api/agents") else { return }
        guard let response: ApiResponse<[ClawAgent]> = decode(data) else { return }
        if response.success, let agents = response.data {
            self.agents = agents
        }
    }

    /// Create a new agent profile.
    func createAgent(id: String, provider: String?, model: String?,
                     apiKey: String?, baseURL: String?, systemPrompt: String?,
                     providerRef: String? = nil) async {
        var body: [String: Any] = ["id": id]
        if let p = provider { body["provider"] = p }
        if let m = model { body["model"] = m }
        if let k = apiKey { body["api_key"] = k }
        if let b = baseURL { body["base_url"] = b }
        if let s = systemPrompt { body["system_prompt"] = s }
        if let r = providerRef { body["provider_ref"] = r }

        guard let bodyData = try? JSONSerialization.data(withJSONObject: body) else { return }
        guard let data = await post("/api/agents", body: bodyData) else { return }
        guard let response: ApiResponse<[String: String]> = decode(data) else { return }
        if response.success {
            await fetchAgents()
        } else if let err = response.error {
            self.errorMessage = err
        }
    }

    /// Delete an agent profile.
    func deleteAgent(_ id: String) async {
        let _ = await delete("/api/agents/\(id)")
        await fetchAgents()
    }

    /// Update an existing agent profile.
    func updateAgent(id: String, provider: String? = nil, model: String? = nil,
                     apiKey: String? = nil, baseURL: String? = nil,
                     systemPrompt: String? = nil, enabledTools: [String]? = nil,
                     providerRef: String? = nil) async -> Bool {
        var body: [String: Any] = [:]
        if let p = provider { body["provider"] = p }
        if let m = model { body["model"] = m }
        if let k = apiKey { body["api_key"] = k }
        if let b = baseURL { body["base_url"] = b }
        if let s = systemPrompt { body["system_prompt"] = s }
        if let tools = enabledTools { body["enabled_tools"] = tools }
        if let r = providerRef { body["provider_ref"] = r }

        guard let bodyData = try? JSONSerialization.data(withJSONObject: body) else { return false }
        guard let data = await put("/api/agents/\(id)", body: bodyData) else { return false }
        guard let response: ApiResponse<[String: String]> = decode(data) else { return false }
        if response.success {
            await fetchAgents()
            return true
        } else if let err = response.error {
            self.errorMessage = err
        }
        return false
    }

    /// Get detailed config for a single agent.
    func getAgentDetail(id: String) async -> AgentDetail? {
        guard let data = await get("/api/agents/\(id)") else { return nil }
        guard let response: ApiResponse<AgentDetail> = decode(data) else { return nil }
        return response.data
    }

    /// Create a new agent profile.
    func createAgent(id: String, provider: String? = nil, model: String? = nil,
                      apiKey: String? = nil, baseURL: String? = nil,
                      systemPrompt: String? = nil, enabledTools: [String]? = nil,
                      providerRef: String? = nil) async -> Bool {
        var body: [String: Any] = ["id": id]
        if let p = provider { body["provider"] = p }
        if let m = model { body["model"] = m }
        if let k = apiKey { body["api_key"] = k }
        if let b = baseURL { body["base_url"] = b }
        if let s = systemPrompt { body["system_prompt"] = s }
        if let tools = enabledTools { body["enabled_tools"] = tools }
        if let r = providerRef { body["provider_ref"] = r }

        guard let bodyData = try? JSONSerialization.data(withJSONObject: body) else { return false }
        guard let data = await post("/api/agents", body: bodyData) else { return false }
        guard let response: ApiResponse<[String: String]> = decode(data) else { return false }
        if response.success {
            await fetchAgents()
            return true
        } else if let err = response.error {
            self.errorMessage = err
        }
        return false
    }

    /// Submit feedback for a session (thumbs up/down).
    func postFeedback(sessionId: String, positive: Bool, message: String? = nil) async {
        var body: [String: Any] = ["positive": positive]
        if let msg = message { body["message"] = msg }

        guard let bodyData = try? JSONSerialization.data(withJSONObject: body) else { return }
        let _ = await post("/api/sessions/\(sessionId)/feedback", body: bodyData)
    }

    // MARK: - Config

    /// Fetch sanitized configuration.
    func fetchConfig() async {
        guard let data = await get("/api/config") else { return }
        guard let response: ApiResponse<ClawConfig> = decode(data) else { return }
        if response.success {
            self.config = response.data
        }
    }

    // MARK: - Tools

    func fetchTools() async {
        guard let data = await get("/api/tools") else { return }
        guard let response: ApiResponse<[ToolInfo]> = decode(data) else { return }
        if response.success, let tools = response.data {
            self.tools = tools
        }
    }

    // MARK: - Skills

    func fetchSkills() async {
        guard let data = await get("/api/skills") else { return }
        guard let response: ApiResponse<[SkillInfo]> = decode(data) else { return }
        if response.success, let skills = response.data {
            self.skills = skills
        }
    }

    // MARK: - Plugins

    func fetchPlugins() async {
        guard let data = await get("/api/plugins") else { return }
        guard let response: ApiResponse<[PluginInfo]> = decode(data) else { return }
        if response.success, let plugins = response.data {
            self.plugins = plugins
        }
    }

    // MARK: - Stats

    func fetchStats(period: String = "all") async {
        isLoadingStats = true
        defer { isLoadingStats = false }

        guard let data = await get("/api/stats?period=\(period)") else { return }
        guard let response: ApiResponse<StatsResponse> = decode(data) else { return }
        if response.success, let stats = response.data {
            self.stats = stats
        }
    }

    // MARK: - HTTP Helpers

    /// Perform a GET request. Uses URLSession.shared for reliability.
    private func get(_ path: String) async -> Data? {
        guard connectionState.isConnected else { return nil }
        let url = URL(string: "\(baseURL)\(path)")!
        var request = URLRequest(url: url)
        request.timeoutInterval = 10
        addAuthHeader(&request)
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse else { return nil }
            if httpResponse.statusCode == 401 {
                self.errorMessage = "认证失败，请在设置中检查 Auth Token"
                return nil
            }
            guard (200...299).contains(httpResponse.statusCode) else {
                return nil
            }
            return data
        } catch {
            self.errorMessage = "Network error: \(error.localizedDescription)"
            return nil
        }
    }

    private func post(_ path: String, body: Data? = nil) async -> Data? {
        guard connectionState.isConnected else { return nil }
        let url = URL(string: "\(baseURL)\(path)")!
        var request = URLRequest(url: url)
        request.httpMethod = "POST"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.httpBody = body
        request.timeoutInterval = 60
        addAuthHeader(&request)
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse else { return nil }
            if httpResponse.statusCode == 401 {
                self.errorMessage = "认证失败，请在设置中检查 Auth Token"
                return nil
            }
            guard (200...299).contains(httpResponse.statusCode) else {
                return nil
            }
            return data
        } catch {
            self.errorMessage = "Network error: \(error.localizedDescription)"
            return nil
        }
    }

    private func delete(_ path: String) async -> Data? {
        guard connectionState.isConnected else { return nil }
        let url = URL(string: "\(baseURL)\(path)")!
        var request = URLRequest(url: url)
        request.httpMethod = "DELETE"
        request.timeoutInterval = 10
        addAuthHeader(&request)
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse else { return nil }
            if httpResponse.statusCode == 401 {
                self.errorMessage = "认证失败，请在设置中检查 Auth Token"
                return nil
            }
            guard (200...299).contains(httpResponse.statusCode) else {
                return nil
            }
            return data
        } catch {
            return nil
        }
    }

    private func put(_ path: String, body: Data? = nil) async -> Data? {
        guard connectionState.isConnected else { return nil }
        let url = URL(string: "\(baseURL)\(path)")!
        var request = URLRequest(url: url)
        request.httpMethod = "PUT"
        request.setValue("application/json", forHTTPHeaderField: "Content-Type")
        request.timeoutInterval = 10
        addAuthHeader(&request)
        if let body { request.httpBody = body }
        do {
            let (data, response) = try await URLSession.shared.data(for: request)
            guard let httpResponse = response as? HTTPURLResponse else { return nil }
            if httpResponse.statusCode == 401 {
                self.errorMessage = "认证失败，请在设置中检查 Auth Token"
                return nil
            }
            guard (200...299).contains(httpResponse.statusCode) else {
                return nil
            }
            return data
        } catch {
            return nil
        }
    }

    // MARK: - Helpers

    private var currentSessionId: String? {
        get { currentSession?.id }
        set {
            if let id = newValue, let session = sessions.first(where: { $0.id == id }) {
                currentSession = session
            } else if newValue == nil {
                currentSession = nil
            }
        }
    }

    private func decode<T: Codable>(_ data: Data) -> T? {
        do {
            return try decoder.decode(T.self, from: data)
        } catch {
            self.errorMessage = "Decode error: \(error.localizedDescription)"
            return nil
        }
    }

    /// Convert a ClawMessage from the API into one or more MessageItems.
    /// Reasoning (DeepSeek chain-of-thought) is emitted as a separate message
    /// before the assistant text, consistent with the SSE streaming model.
    private func convertToAppMessages(_ msg: ClawMessage) -> [MessageItem] {
        var items: [MessageItem] = []
        // Emit reasoning as a separate message if present
        if let reasoning = msg.reasoning, !reasoning.isEmpty {
            items.append(MessageItem(message: .reasoning(text: reasoning)))
        }
        let appMsg: AppMessage
        switch msg.role {
        case "user":
            appMsg = .user(text: msg.content ?? "")
        case "assistant":
            appMsg = .assistant(text: msg.content ?? "")
        case "tool_call":
            appMsg = .toolCall(name: msg.name ?? "", args: msg.args ?? "", result: msg.result ?? "")
        case "evaluation":
            appMsg = .evaluation(tool: msg.tool ?? "", valid: msg.valid ?? false, issues: msg.issues ?? [])
        case "quality":
            appMsg = .quality(score: msg.score ?? "", complete: msg.complete ?? false, issues: msg.issues ?? [], referencesValid: msg.referencesValid ?? false)
        default:
            appMsg = .assistant(text: msg.content ?? "")
        }
        items.append(MessageItem(message: appMsg))
        return items
    }

}


