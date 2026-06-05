import Foundation

// MARK: - Generic API Response

struct ApiResponse<T: Codable>: Codable {
    let success: Bool
    let data: T?
    let error: String?
}

// MARK: - Session

struct ClawSession: Codable, Identifiable, Hashable {
    let id: String
    var title: String
    let messageCount: Int
    let createdAt: Int64?
    let agentId: String?

    enum CodingKeys: String, CodingKey {
        case id, title
        case messageCount = "message_count"
        case createdAt = "created_at"
        case agentId = "agent_id"
    }

    func hash(into hasher: inout Hasher) {
        hasher.combine(id)
    }

    static func == (lhs: ClawSession, rhs: ClawSession) -> Bool {
        lhs.id == rhs.id
    }

    var formattedDate: String {
        guard let ts = createdAt else { return "" }
        let date = Date(timeIntervalSince1970: TimeInterval(ts))
        let fmt = DateFormatter()
        fmt.dateStyle = .short
        fmt.timeStyle = .short
        return fmt.string(from: date)
    }

    var dateValue: Date {
        guard let ts = createdAt else { return .distantPast }
        return Date(timeIntervalSince1970: TimeInterval(ts))
    }

    var shortDate: String {
        let date = dateValue
        let calendar = Calendar.current
        let fmt = DateFormatter()
        if calendar.isDateInToday(date) {
            fmt.dateFormat = "HH:mm"
        } else if calendar.isDateInYesterday(date) {
            return "Yesterday"
        } else {
            fmt.dateFormat = "MM-dd"
        }
        return fmt.string(from: date)
    }
}

// MARK: - Message (from API)

struct ClawMessage: Codable, Identifiable {
    let id = UUID()
    let role: String
    let content: String?
    let reasoning: String?
    let name: String?
    let args: String?
    let result: String?
    // evaluation fields
    let tool: String?
    let valid: Bool?
    let issues: [String]?
    // quality fields
    let score: String?
    let complete: Bool?
    let referencesValid: Bool?

    enum CodingKeys: String, CodingKey {
        case role, content, reasoning, name, args, result, tool, valid, issues, score, complete, referencesValid
    }

    var displayContent: String {
        if role == "tool_call" {
            return "🛠 \(name ?? "tool")"
        }
        return content ?? ""
    }
}

struct SessionDetail: Codable {
    let id: String
    let title: String?
    let messages: [ClawMessage]
    let agentId: String?

    enum CodingKeys: String, CodingKey {
        case id, title, messages
        case agentId = "agent_id"
    }
}

// MARK: - Agent

struct ClawAgent: Codable, Identifiable, Hashable {
    let id: String
    let provider: String?
    let model: String?
    let baseUrl: String?
    let toolCount: Int?
    let enabledTools: [String]?
    let systemPrompt: String?
    let isSubAgent: Bool?
    let providerRef: String?

    enum CodingKeys: String, CodingKey {
        case id, provider, model
        case baseUrl = "base_url"
        case toolCount = "tool_count"
        case enabledTools = "enabled_tools"
        case systemPrompt = "system_prompt"
        case isSubAgent = "is_sub_agent"
        case providerRef = "provider_ref"
    }
}

struct AgentDetail: Codable {
    let id: String
    let provider: String
    let model: String
    let baseUrl: String
    let enabledTools: [String]?
    let toolCount: Int
    let systemPrompt: String?
    let mcpServers: [String]?
    let allowedDirs: [String]?
    let providerRef: String?

    enum CodingKeys: String, CodingKey {
        case id, provider, model
        case baseUrl = "base_url"
        case enabledTools = "enabled_tools"
        case toolCount = "tool_count"
        case systemPrompt = "system_prompt"
        case mcpServers = "mcp_servers"
        case allowedDirs = "allowed_dirs"
        case providerRef = "provider_ref"
    }
}

// MARK: - Config

struct ClawConfig: Codable {
    let provider: String?
    let model: String?
    let enabledTools: [String]?
    let mcpServers: [MCPServerConfig]?
    let pluginsAutoDiscover: Bool?
    let providers: [String: ProviderConfig]?
    let defaultProvider: String?

    enum CodingKeys: String, CodingKey {
        case provider, model
        case enabledTools = "enabled_tools"
        case mcpServers = "mcp_servers"
        case pluginsAutoDiscover = "plugins_auto_discover"
        case providers
        case defaultProvider = "default_provider"
    }
}

struct MCPServerConfig: Codable {
    let name: String
    let command: String?
    let args: [String]?
    let url: String?
}

/// A named provider configuration matching [providers] in config.toml
struct ProviderConfig: Codable {
    let provider: String
    let apiKey: String
    let baseUrl: String
    let model: String

    enum CodingKeys: String, CodingKey {
        case provider
        case apiKey = "api_key"
        case baseUrl = "base_url"
        case model
    }
}

// MARK: - Tool

struct ToolInfo: Codable, Identifiable {
    let id = UUID()
    let type: String
    let name: String
    let description: String

    enum CodingKeys: String, CodingKey {
        case type
        case name = "name"
        case description = "description"
        case function
    }

    let function: ToolFunction?

    struct ToolFunction: Codable {
        let name: String
        let description: String
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        type = try container.decodeIfPresent(String.self, forKey: .type) ?? "function"

        if let fn = try? container.decodeIfPresent(ToolFunction.self, forKey: .function) {
            function = fn
            name = fn.name
            description = fn.description
        } else {
            function = nil
            name = (try? container.decodeIfPresent(String.self, forKey: .name)) ?? ""
            description = (try? container.decodeIfPresent(String.self, forKey: .description)) ?? ""
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.container(keyedBy: CodingKeys.self)
        try container.encode(type, forKey: .type)
        try container.encode(name, forKey: .name)
        try container.encode(description, forKey: .description)
    }
}

// MARK: - Skill

struct SkillInfo: Codable, Identifiable {
    var id = UUID()
    let name: String
    let description: String
    let content: String
}

// MARK: - Plugin

struct PluginInfo: Codable, Identifiable {
    var id = UUID()
    let name: String
    let version: String
    let description: String
    let author: String?
    let enabled: Bool
}

// MARK: - Chat Send Response

struct ChatResponse: Codable {
    let sessionId: String
    let status: String

    enum CodingKeys: String, CodingKey {
        case sessionId = "session_id"
        case status
    }
}

// MARK: - SSE Event Types

enum SseEvent {
    case token(String)
    case reasoning(String)
    case status(String)
    case toolExecuted(name: String, args: String, result: String, step: Int, totalSteps: Int)
    case newRound
    case done(usage: TokenUsage?)
    case evaluation(tool: String, valid: Bool, issues: [String])
    case qualityScore(score: String, complete: Bool, issues: [String], referencesValid: Bool)
    case error(String)
}

struct TokenUsage: Codable {
    let promptTokens: Int?
    let completionTokens: Int?
    let estimatedCostUsd: Double?

    enum CodingKeys: String, CodingKey {
        case promptTokens = "prompt_tokens"
        case completionTokens = "completion_tokens"
        case estimatedCostUsd = "estimated_cost_usd"
    }

    var totalTokens: Int {
        (promptTokens ?? 0) + (completionTokens ?? 0)
    }

    /// Formatted token string: ↑prompt ↓completion
    var formattedTokens: String {
        let p = promptTokens ?? 0
        let c = completionTokens ?? 0
        let fmt = { (n: Int) -> String in
            if n >= 1000 { return String(format: "%.1fk", Double(n) / 1000.0) }
            return "\(n)"
        }
        return "↑\(fmt(p)) ↓\(fmt(c))"
    }

    /// Total estimated cost in USD
    var totalCost: Double? {
        estimatedCostUsd
    }

    /// Formatted cost string
    var formattedCost: String? {
        guard let cost = estimatedCostUsd, cost > 0 else { return nil }
        if cost < 0.01 { return String(format: "$%.4f", cost) }
        return String(format: "$%.3f", cost)
    }
}

struct TokenStats: Codable {
    let requests: Int?
    let tokens: Int?
    let costUsd: Double?

    enum CodingKeys: String, CodingKey {
        case requests, tokens
        case costUsd = "cost_usd"
    }
}

struct StatsResponse: Codable {
    let totalRequests: Int?
    let totalTokens: Int?
    let totalCostUsd: Double?
    let today: TokenStats?

    enum CodingKeys: String, CodingKey {
        case totalRequests = "total_requests"
        case totalTokens = "total_tokens"
        case totalCostUsd = "total_cost_usd"
        case today
    }
}

extension TokenUsage {
    static func + (lhs: TokenUsage, rhs: TokenUsage) -> TokenUsage {
        TokenUsage(
            promptTokens: (lhs.promptTokens ?? 0) + (rhs.promptTokens ?? 0),
            completionTokens: (lhs.completionTokens ?? 0) + (rhs.completionTokens ?? 0),
            estimatedCostUsd: (lhs.estimatedCostUsd ?? 0) + (rhs.estimatedCostUsd ?? 0)
        )
    }
}

// MARK: - Backend Config

struct BackendConfig: Identifiable, Codable, Equatable {
    let id: UUID
    var name: String
    var url: String
    var authToken: String

    init(id: UUID = UUID(), name: String, url: String, authToken: String = "") {
        self.id = id
        self.name = name
        self.url = url
        self.authToken = authToken
    }
}

// MARK: - UI Message Types

/// Message item with a stable identity for SwiftUI `ForEach`.
struct MessageItem: Identifiable {
    let id: UUID
    let message: AppMessage
    var tokenUsage: TokenUsage?

    init(id: UUID = UUID(), message: AppMessage, tokenUsage: TokenUsage? = nil) {
        self.id = id
        self.message = message
        self.tokenUsage = tokenUsage
    }
}

enum AppMessage {
    case user(text: String)
    case assistant(text: String)
    case toolCall(name: String, args: String, result: String)
    case error(text: String)
    case status(text: String)
    case reasoning(text: String)
    case evaluation(tool: String, valid: Bool, issues: [String])
    case quality(score: String, complete: Bool, issues: [String], referencesValid: Bool)

    var text: String {
        switch self {
        case .user(let t): return t
        case .assistant(let t): return t
        case .toolCall(let n, _, _): return "🛠 \(n)"
        case .error(let t): return t
        case .status(let t): return t
        case .reasoning(let t): return t
        case .evaluation(let tool, let valid, _): return "📋 \(tool): \(valid ? "✓" : "✗")"
        case .quality(let score, _, _, _): return "⭐ 质量评分: \(score)"
        }
    }

    var isAssistant: Bool {
        if case .assistant = self { return true }
        return false
    }
}
