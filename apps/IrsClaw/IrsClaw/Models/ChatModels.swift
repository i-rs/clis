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
    let name: String?
    let args: String?
    let result: String?

    enum CodingKeys: String, CodingKey {
        case role, content, name, args, result
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

    enum CodingKeys: String, CodingKey {
        case id, provider, model
        case baseUrl = "base_url"
        case toolCount = "tool_count"
        case enabledTools = "enabled_tools"
        case systemPrompt = "system_prompt"
    }
}

// MARK: - Config

struct ClawConfig: Codable {
    let provider: String?
    let model: String?
    let enabledTools: [String]?
    let mcpServers: [MCPServerConfig]?
    let pluginsAutoDiscover: Bool?

    enum CodingKeys: String, CodingKey {
        case provider, model
        case enabledTools = "enabled_tools"
        case mcpServers = "mcp_servers"
        case pluginsAutoDiscover = "plugins_auto_discover"
    }
}

struct MCPServerConfig: Codable {
    let name: String
    let command: String?
    let args: [String]?
    let url: String?
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
    case error(String)
}

struct TokenUsage: Codable {
    let promptTokens: Int?
    let completionTokens: Int?

    enum CodingKeys: String, CodingKey {
        case promptTokens = "prompt_tokens"
        case completionTokens = "completion_tokens"
    }
}

// MARK: - UI Message Types

/// Message item with a stable identity for SwiftUI `ForEach`.
struct MessageItem: Identifiable {
    let id = UUID()
    let message: AppMessage
}

enum AppMessage {
    case user(text: String)
    case assistant(text: String)
    case toolCall(name: String, args: String, result: String)
    case error(text: String)
    case status(text: String)
    case reasoning(text: String)

    var text: String {
        switch self {
        case .user(let t): return t
        case .assistant(let t): return t
        case .toolCall(let n, _, _): return "🛠 \(n)"
        case .error(let t): return t
        case .status(let t): return t
        case .reasoning(let t): return t
        }
    }
}
