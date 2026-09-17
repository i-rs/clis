package me.siwi.irsclaw.ui.chat

import androidx.compose.animation.animateContentSize
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ExpandLess
import androidx.compose.material.icons.filled.ExpandMore
import androidx.compose.material.icons.filled.Psychology
import androidx.compose.material.icons.filled.ThumbDown
import androidx.compose.material.icons.filled.ThumbUp
import androidx.compose.material.icons.outlined.ThumbDown
import androidx.compose.material.icons.outlined.ThumbUp
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.unit.dp
import coil.compose.AsyncImage
import me.siwi.irsclaw.data.model.AppMessage
import me.siwi.irsclaw.data.model.MessageItem
import me.siwi.irsclaw.ui.components.AvatarView
import me.siwi.irsclaw.ui.components.MarkdownText
import me.siwi.irsclaw.ui.components.PulsingDot
import me.siwi.irsclaw.ui.theme.MonoStyle

/** One chat bubble of any of the 10 supported types (iOS MessageBubbleView). */
@Composable
fun MessageBubble(
    item: MessageItem,
    onFeedback: (positive: Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    when (val message = item.message) {
        is AppMessage.User -> UserBubble(message.text, modifier)
        is AppMessage.Assistant -> AssistantBubble(
            text = message.text,
            usage = item.tokenUsage,
            onFeedback = onFeedback,
            modifier = modifier,
        )
        is AppMessage.ToolCall -> Row(
            modifier = modifier.padding(horizontal = 16.dp, vertical = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            AvatarView(message)
            ToolCallCard(message)
        }
        is AppMessage.Reasoning -> ReasoningBlock(message.text, modifier)
        is AppMessage.Status -> StatusLine(message.text, modifier)
        is AppMessage.Error -> ErrorCard(message.text, modifier)
        is AppMessage.Evaluation -> EvaluationCard(message, modifier)
        is AppMessage.Quality -> QualityCard(message, modifier)
        is AppMessage.Feedback -> FeedbackEcho(message, modifier)
        is AppMessage.Image -> ImageBubble(message, modifier)
    }
}

// MARK: - User

@Composable
private fun UserBubble(text: String, modifier: Modifier = Modifier) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp),
        horizontalArrangement = Arrangement.End,
    ) {
        Text(
            text = text,
            style = MaterialTheme.typography.bodyLarge,
            color = MaterialTheme.colorScheme.onPrimary,
            modifier = Modifier
                .widthIn(max = 300.dp)
                .clip(RoundedCornerShape(18.dp))
                .background(MaterialTheme.colorScheme.primary)
                .padding(horizontal = 14.dp, vertical = 10.dp),
        )
    }
}

// MARK: - Assistant (markdown + usage badge + thumbs)

@Composable
private fun AssistantBubble(
    text: String,
    usage: me.siwi.irsclaw.data.model.TokenUsage?,
    onFeedback: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    var feedbackSent by rememberSaveable { mutableStateOf(false) }
    var positiveVote by rememberSaveable { mutableStateOf(true) }
    val scheme = MaterialTheme.colorScheme

    Column(modifier = modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 4.dp)) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
            AvatarView(AppMessage.Assistant(""))
            Column(
                modifier = Modifier
                    .widthIn(max = 320.dp)
                    .clip(RoundedCornerShape(16.dp))
                    .background(scheme.surfaceContainer)
                    .padding(horizontal = 12.dp, vertical = 8.dp),
            ) {
                MarkdownText(markdown = text)
            }
        }

        // Usage capsule + one-shot thumbs row (iOS shows both under each assistant bubble).
        Row(
            modifier = Modifier.padding(start = 36.dp, top = 4.dp),
            horizontalArrangement = Arrangement.spacedBy(10.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            if (usage != null) {
                val capsule = buildString {
                    append(usage.formattedTokens)
                    usage.formattedCost?.let { append(" · $it") }
                }
                Text(
                    text = capsule,
                    style = MonoStyle,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier
                        .clip(RoundedCornerShape(50))
                        .background(scheme.surfaceContainerHigh)
                        .padding(horizontal = 8.dp, vertical = 2.dp),
                )
            }
            if (!feedbackSent) {
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Icon(
                        Icons.Outlined.ThumbUp,
                        contentDescription = "赞",
                        tint = scheme.onSurfaceVariant,
                        modifier = Modifier
                            .size(16.dp)
                            .clickable {
                                feedbackSent = true
                                positiveVote = true
                                onFeedback(true)
                            },
                    )
                    Icon(
                        Icons.Outlined.ThumbDown,
                        contentDescription = "踩",
                        tint = scheme.onSurfaceVariant,
                        modifier = Modifier
                            .size(16.dp)
                            .clickable {
                                feedbackSent = true
                                positiveVote = false
                                onFeedback(false)
                            },
                    )
                }
            }
        }
    }
}

// MARK: - Reasoning (collapsible "Thinking" block)

@Composable
private fun ReasoningBlock(text: String, modifier: Modifier = Modifier) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(scheme.surfaceContainer.copy(alpha = 0.7f))
            .animateContentSize(),
    ) {
        Row(
            modifier = Modifier
                .fillMaxWidth()
                .clickable { expanded = !expanded }
                .padding(horizontal = 12.dp, vertical = 8.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            if (!expanded) PulsingDot(size = 7.dp, color = Color(0xFF5E5CE6))
            Icon(
                Icons.Filled.Psychology,
                contentDescription = null,
                tint = Color(0xFF5E5CE6),
                modifier = Modifier.size(16.dp),
            )
            Text(
                text = "思考中 · ${text.length} 字符",
                style = MaterialTheme.typography.labelMedium,
                color = scheme.onSurfaceVariant,
            )
            Spacer(Modifier.weight(1f))
            Icon(
                if (expanded) Icons.Filled.ExpandLess else Icons.Filled.ExpandMore,
                contentDescription = null,
                tint = scheme.onSurfaceVariant,
                modifier = Modifier.size(16.dp),
            )
        }
        if (expanded) {
            Text(
                text = text,
                style = MaterialTheme.typography.bodySmall,
                color = scheme.onSurfaceVariant,
                modifier = Modifier.padding(start = 12.dp, end = 12.dp, bottom = 10.dp),
            )
        }
    }
}

// MARK: - Status / Error / Evaluation / Quality / Feedback / Image

@Composable
private fun StatusLine(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = 4.dp),
        textAlign = androidx.compose.ui.text.style.TextAlign.Center,
    )
}

@Composable
private fun ErrorCard(text: String, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    Text(
        text = "⚠️ $text",
        style = MaterialTheme.typography.bodyMedium,
        color = scheme.error,
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(scheme.error.copy(alpha = 0.1f))
            .padding(12.dp),
    )
}

@Composable
private fun EvaluationCard(eval: AppMessage.Evaluation, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(scheme.surfaceContainer)
            .padding(12.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(text = "📋 评估", style = MaterialTheme.typography.labelLarge, color = scheme.onSurface)
            Text(
                text = "${eval.tool} ${if (eval.valid) "✓" else "✗"}",
                style = MonoStyle,
                color = if (eval.valid) Color(0xFF30D158) else scheme.error,
            )
        }
        if (eval.issues.isNotEmpty()) {
            eval.issues.forEach { issue ->
                Text(
                    text = "· $issue",
                    style = MaterialTheme.typography.bodySmall,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 2.dp),
                )
            }
        }
    }
}

@Composable
private fun QualityCard(quality: AppMessage.Quality, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .clip(RoundedCornerShape(12.dp))
            .background(scheme.surfaceContainer)
            .padding(12.dp),
    ) {
        Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
            Text(text = "⭐ 质量评分", style = MaterialTheme.typography.labelLarge, color = scheme.onSurface)
            Text(text = quality.score, style = MonoStyle, color = scheme.primary)
            Text(
                text = if (quality.complete) "完整" else "未完整",
                style = MaterialTheme.typography.labelMedium,
                color = if (quality.complete) Color(0xFF30D158) else Color(0xFFFF9500),
            )
        }
        if (quality.issues.isNotEmpty()) {
            quality.issues.forEach { issue ->
                Text(
                    text = "· $issue",
                    style = MaterialTheme.typography.bodySmall,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 2.dp),
                )
            }
        }
    }
}

@Composable
private fun FeedbackEcho(feedback: AppMessage.Feedback, modifier: Modifier = Modifier) {
    Text(
        text = feedback.preview,
        style = MaterialTheme.typography.labelMedium,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        modifier = modifier.fillMaxWidth().padding(vertical = 4.dp),
        textAlign = androidx.compose.ui.text.style.TextAlign.Center,
    )
}

@Composable
private fun ImageBubble(image: AppMessage.Image, modifier: Modifier = Modifier) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 4.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        AsyncImage(
            model = image.url,
            contentDescription = image.altText,
            contentScale = ContentScale.FillWidth,
            modifier = Modifier
                .fillMaxWidth()
                .widthIn(max = 300.dp)
                .clip(RoundedCornerShape(12.dp)),
        )
        if (image.altText.isNotEmpty()) {
            Text(
                text = "${image.altText} · ${image.width}×${image.height} ${image.format}",
                style = MaterialTheme.typography.labelSmall,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier.padding(top = 4.dp),
            )
        }
    }
}
