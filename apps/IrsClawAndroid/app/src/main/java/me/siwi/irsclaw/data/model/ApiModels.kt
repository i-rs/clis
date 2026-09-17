package me.siwi.irsclaw.data.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

// MARK: - Generic API Response

@Serializable
data class ApiResponse<T>(
    val success: Boolean = false,
    val data: T? = null,
    val error: String? = null,
)

// MARK: - Session

@Serializable
data class SessionMeta(
    val id: String,
    val title: String = "",
    @SerialName("message_count") val messageCount: Int = 0,
    @SerialName("created_at") val createdAt: Long? = null,
    @SerialName("agent_id") val agentId: String? = null,
)

@Serializable
data class ClawMessage(
    val role: String,
    val content: String? = null,
    val reasoning: String? = null,
    val name: String? = null,
    val args: String? = null,
    val result: String? = null,
    // evaluation fields
    val tool: String? = null,
    val valid: Boolean? = null,
    val issues: List<String>? = null,
    // quality fields
    val score: String? = null,
    val complete: Boolean? = null,
    @SerialName("references_valid") val referencesValid: Boolean? = null,
    // feedback fields
    val positive: Boolean? = null,
    val message: String? = null,
    // image fields
    @SerialName("alt_text") val altText: String? = null,
    val width: Int? = null,
    val height: Int? = null,
    val format: String? = null,
    val url: String? = null,
)

@Serializable
data class CurrentSession(
    val id: String? = null,
    val title: String? = null,
    @SerialName("message_count") val messageCount: Int = 0,
    @SerialName("agent_id") val agentId: String? = null,
    val messages: List<ClawMessage> = emptyList(),
)

@Serializable
data class SessionDetail(
    val id: String,
    val title: String? = null,
    val messages: List<ClawMessage> = emptyList(),
    @SerialName("agent_id") val agentId: String? = null,
)

// MARK: - Agent

@Serializable
data class ClawAgent(
    val id: String,
    val provider: String? = null,
    val model: String? = null,
    @SerialName("base_url") val baseUrl: String? = null,
    @SerialName("tool_count") val toolCount: Int = 0,
    @SerialName("enabled_tools") val enabledTools: List<String> = emptyList(),
    @SerialName("system_prompt") val systemPrompt: String? = null,
    @SerialName("is_sub_agent") val isSubAgent: Boolean? = null,
    @SerialName("provider_ref") val providerRef: String? = null,
)

@Serializable
data class AgentDetail(
    val id: String,
    val provider: String? = null,
    val model: String? = null,
    @SerialName("base_url") val baseUrl: String? = null,
    @SerialName("enabled_tools") val enabledTools: List<String> = emptyList(),
    @SerialName("tool_count") val toolCount: Int = 0,
    @SerialName("system_prompt") val systemPrompt: String? = null,
    @SerialName("mcp_servers") val mcpServers: List<String>? = null,
    @SerialName("allowed_dirs") val allowedDirs: List<String>? = null,
    @SerialName("provider_ref") val providerRef: String? = null,
)

@Serializable
data class AgentUpsertRequest(
    val id: String? = null,
    val provider: String? = null,
    val model: String? = null,
    @SerialName("api_key") val apiKey: String? = null,
    @SerialName("base_url") val baseUrl: String? = null,
    @SerialName("system_prompt") val systemPrompt: String? = null,
    @SerialName("enabled_tools") val enabledTools: List<String>? = null,
    @SerialName("provider_ref") val providerRef: String? = null,
)

// MARK: - Config

@Serializable
data class ClawConfig(
    val provider: String? = null,
    val model: String? = null,
    @SerialName("enabled_tools") val enabledTools: List<String>? = null,
    @SerialName("mcp_servers") val mcpServers: List<McpServerInfo> = emptyList(),
    @SerialName("plugins_auto_discover") val pluginsAutoDiscover: Boolean? = null,
    val providers: Map<String, ProviderEntry> = emptyMap(),
    @SerialName("default_provider") val defaultProvider: String? = null,
)

@Serializable
data class McpServerInfo(
    val name: String,
    val command: String? = null,
    val args: List<String> = emptyList(),
    val url: String? = null,
)

@Serializable
data class ProviderEntry(
    val provider: String? = null,
    @SerialName("api_key") val apiKey: String? = null,
    @SerialName("base_url") val baseUrl: String? = null,
    val model: String? = null,
)

/** PATCH /api/config body for switching the LLM provider. */
@Serializable
data class LlmConfigPatch(
    val provider: String,
    @SerialName("api_key") val apiKey: String,
    @SerialName("base_url") val baseUrl: String,
    val model: String? = null,
)

// MARK: - Tool (OpenAI-style schema with flat fallback)

@Serializable(with = ToolInfoSerializer::class)
data class ToolInfo(
    val name: String,
    val description: String,
)

// MARK: - Skill / Plugin

@Serializable
data class SkillInfo(
    val name: String,
    val description: String = "",
    val content: String = "",
    val parameters: kotlinx.serialization.json.JsonElement? = null,
)

@Serializable
data class PluginInfo(
    val name: String,
    val version: String = "",
    val description: String = "",
    val author: String? = null,
    val enabled: Boolean = false,
)

// MARK: - Stats

@Serializable
data class TokenStats(
    val requests: Int = 0,
    val tokens: Int = 0,
    @SerialName("cost_usd") val costUsd: Double = 0.0,
)

@Serializable
data class StatsResponse(
    @SerialName("total_requests") val totalRequests: Long = 0,
    @SerialName("total_tokens") val totalTokens: Long = 0,
    @SerialName("total_cost_usd") val totalCostUsd: Double = 0.0,
    val today: TokenStats? = null,
)

// MARK: - SSE payload shapes

@Serializable
data class TokenUsage(
    @SerialName("prompt_tokens") val promptTokens: Int = 0,
    @SerialName("completion_tokens") val completionTokens: Int = 0,
    @SerialName("total_tokens") val serverTotalTokens: Int = 0,
    @SerialName("estimated_cost_usd") val estimatedCostUsd: Double = 0.0,
) {
    val totalTokens: Int get() = promptTokens + completionTokens

    /** "↑1.2k ↓345" style summary, matching the iOS client. */
    val formattedTokens: String
        get() {
            fun fmt(n: Int): String = if (n >= 1000) "%.1fk".format(n / 1000.0) else n.toString()
            return "↑${fmt(promptTokens)} ↓${fmt(completionTokens)}"
        }

    val formattedCost: String?
        get() = when {
            estimatedCostUsd <= 0.0 -> null
            estimatedCostUsd < 0.01 -> "$%.4f".format(estimatedCostUsd)
            else -> "$%.3f".format(estimatedCostUsd)
        }

    operator fun plus(other: TokenUsage): TokenUsage = TokenUsage(
        promptTokens = promptTokens + other.promptTokens,
        completionTokens = completionTokens + other.completionTokens,
        serverTotalTokens = serverTotalTokens + other.serverTotalTokens,
        estimatedCostUsd = estimatedCostUsd + other.estimatedCostUsd,
    )
}

@Serializable
data class QualityInfo(
    val score: String = "",
    val complete: Boolean = false,
    val issues: List<String> = emptyList(),
    @SerialName("references_valid") val referencesValid: Boolean? = null,
)

@Serializable
data class DonePayload(
    val usage: TokenUsage? = null,
    val quality: QualityInfo? = null,
    @SerialName("session_id") val sessionId: String? = null,
)

@Serializable
data class ToolExecutedPayload(
    val name: String = "",
    val args: String = "",
    val result: String = "",
    val step: Int = 0,
    @SerialName("total_steps") val totalSteps: Int = 0,
)

@Serializable
data class EvaluationPayload(
    val tool: String = "",
    val valid: Boolean = false,
    val issues: List<String> = emptyList(),
)

@Serializable
data class ImageGeneratedPayload(
    val path: String = "",
    @SerialName("alt_text") val altText: String = "",
    val format: String = "png",
    val width: Int = 0,
    val height: Int = 0,
)

// MARK: - Request bodies

@Serializable
data class ChatRequest(
    val message: String,
    @SerialName("agent_id") val agentId: String? = null,
)

@Serializable
data class FeedbackRequest(
    val positive: Boolean,
    val message: String? = null,
)
