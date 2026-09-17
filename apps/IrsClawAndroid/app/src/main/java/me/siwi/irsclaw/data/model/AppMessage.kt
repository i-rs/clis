package me.siwi.irsclaw.data.model

import java.util.UUID

/** The 10 chat bubble types, mirroring the iOS `AppMessage` enum. */
sealed interface AppMessage {
    val preview: String

    data class User(val text: String) : AppMessage {
        override val preview: String get() = text
    }

    data class Assistant(val text: String) : AppMessage {
        override val preview: String get() = text
    }

    data class ToolCall(
        val name: String,
        val args: String,
        val result: String,
        val step: Int,
        val totalSteps: Int,
    ) : AppMessage {
        override val preview: String get() = "🛠 $name"
    }

    data class Error(val text: String) : AppMessage {
        override val preview: String get() = text
    }

    data class Status(val text: String) : AppMessage {
        override val preview: String get() = text
    }

    data class Reasoning(val text: String) : AppMessage {
        override val preview: String get() = text
    }

    data class Evaluation(val tool: String, val valid: Boolean, val issues: List<String>) : AppMessage {
        override val preview: String get() = "📋 $tool: ${if (valid) "✓" else "✗"}"
    }

    data class Quality(
        val score: String,
        val complete: Boolean,
        val issues: List<String>,
        val referencesValid: Boolean,
    ) : AppMessage {
        override val preview: String get() = "⭐ 质量评分: $score"
    }

    data class Feedback(val positive: Boolean, val message: String?) : AppMessage {
        override val preview: String
            get() = "💬 反馈: ${if (positive) "👍" else "👎"}${message?.let { " - $it" } ?: ""}"
    }

    data class Image(
        val path: String,
        val altText: String,
        val width: Int,
        val height: Int,
        val format: String,
        val url: String,
    ) : AppMessage {
        override val preview: String get() = "🖼 $altText"
    }
}

/** Chat bubble with a stable identity for LazyColumn item keys. */
data class MessageItem(
    val id: String = UUID.randomUUID().toString(),
    val message: AppMessage,
    val tokenUsage: TokenUsage? = null,
) {
    val isAssistant: Boolean get() = message is AppMessage.Assistant
}

/** Maps a persisted history message into bubble items (reasoning first, mirroring iOS). */
fun ClawMessage.toMessageItems(): List<MessageItem> {
    val items = mutableListOf<MessageItem>()
    reasoning?.takeIf { it.isNotEmpty() }?.let {
        items += MessageItem(message = AppMessage.Reasoning(it))
    }
    val appMessage: AppMessage = when (role) {
        "user" -> AppMessage.User(content ?: "")
        "assistant" -> AppMessage.Assistant(content ?: "")
        "tool_call" -> AppMessage.ToolCall(
            name = name ?: "",
            args = args ?: "",
            result = result ?: "",
            step = 0,
            totalSteps = 0,
        )
        "evaluation" -> AppMessage.Evaluation(tool ?: "", valid ?: false, issues ?: emptyList())
        "quality" -> AppMessage.Quality(
            score = score ?: "",
            complete = complete ?: false,
            issues = issues ?: emptyList(),
            referencesValid = referencesValid ?: false,
        )
        "feedback" -> AppMessage.Feedback(positive = positive ?: true, message = message)
        "image" -> AppMessage.Image(
            path = url ?: "",
            altText = altText ?: "",
            width = width ?: 0,
            height = height ?: 0,
            format = format ?: "",
            url = url ?: "",
        )
        else -> AppMessage.Assistant(content ?: "")
    }
    items += MessageItem(message = appMessage)
    return items
}

fun List<ClawMessage>.toMessageItems(): List<MessageItem> = flatMap { it.toMessageItems() }
