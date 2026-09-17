package me.siwi.irsclaw.ui.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Functions
import androidx.compose.material.icons.filled.Photo
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.ThumbDown
import androidx.compose.material.icons.filled.ThumbUp
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material.icons.outlined.ThumbDown
import androidx.compose.material.icons.outlined.ThumbUp
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import coil.compose.SubcomposeAsyncImage
import me.siwi.irsclaw.data.model.AppMessage
import me.siwi.irsclaw.data.model.MessageItem
import me.siwi.irsclaw.ui.components.AvatarView
import me.siwi.irsclaw.ui.components.MarkdownText
import me.siwi.irsclaw.ui.theme.IosColors

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
        is AppMessage.ToolCall -> WithAvatar(message, modifier) { ToolCallCard(message) }
        is AppMessage.Reasoning -> WithAvatar(message, modifier) { ReasoningBlock(message.text) }
        is AppMessage.Status -> StatusLine(message.text, modifier)
        is AppMessage.Error -> ErrorCard(message.text, modifier)
        is AppMessage.Evaluation -> EvaluationCard(message, modifier)
        is AppMessage.Quality -> QualityCard(message, modifier)
        is AppMessage.Feedback -> FeedbackCard(message, modifier)
        is AppMessage.Image -> ImageBubble(message, modifier)
    }
}

/** Avatar left + content, HStack(top, 10), right spacer(20), vertical padding 2 (iOS row pattern). */
@Composable
private fun WithAvatar(message: AppMessage, modifier: Modifier = Modifier, content: @Composable () -> Unit) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 2.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        AvatarView(message)
        Column(Modifier.weight(1f, fill = false)) {
            content()
        }
        Spacer(Modifier.width(20.dp))
    }
}

// MARK: - User (flat accent bubble, avatar on the right)

@Composable
private fun UserBubble(text: String, modifier: Modifier = Modifier) {
    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 2.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Spacer(Modifier.weight(1f))
        Text(
            text = text,
            style = MaterialTheme.typography.bodyLarge,
            color = Color.White,
            modifier = Modifier
                .widthIn(max = 320.dp)
                .clip(RoundedCornerShape(18.dp))
                .background(MaterialTheme.colorScheme.primary)
                .padding(horizontal = 16.dp, vertical = 12.dp),
        )
        AvatarView(AppMessage.User(""))
    }
}

// MARK: - Assistant (filled secondary card + usage capsule + one-shot thumbs)

@Composable
private fun AssistantBubble(
    text: String,
    usage: me.siwi.irsclaw.data.model.TokenUsage?,
    onFeedback: (Boolean) -> Unit,
    modifier: Modifier = Modifier,
) {
    var feedbackSent by rememberSaveable { mutableStateOf(false) }
    val scheme = MaterialTheme.colorScheme

    Row(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 2.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        AvatarView(AppMessage.Assistant(""))
        Column(Modifier.weight(1f, fill = false)) {
            Box {
                Column(
                    modifier = Modifier
                        .widthIn(max = 320.dp)
                        .clip(RoundedCornerShape(18.dp))
                        .background(scheme.surfaceVariant)
                        .padding(14.dp),
                ) {
                    MarkdownText(markdown = text)
                }
                if (usage != null) {
                    Text(
                        text = buildString {
                            append(usage.formattedTokens)
                            usage.formattedCost?.let { append(" · $it") }
                        },
                        style = MaterialTheme.typography.labelSmall.copy(fontSize = 9.sp),
                        color = scheme.onSurfaceVariant,
                        modifier = Modifier
                            .align(Alignment.BottomEnd)
                            .padding(end = 8.dp, bottom = 4.dp)
                            .background(scheme.onSurface.copy(alpha = 0.06f), RoundedCornerShape(50))
                            .padding(horizontal = 6.dp, vertical = 2.dp),
                    )
                }
            }

            // One-shot thumbs row; after voting, a thanks note (iOS behavior).
            if (!feedbackSent) {
                Row(
                    modifier = Modifier.padding(start = 4.dp, top = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    Icon(
                        Icons.Outlined.ThumbUp,
                        contentDescription = "Thumbs up",
                        tint = scheme.onSurfaceVariant,
                        modifier = Modifier
                            .size(12.dp)
                            .clickable {
                                feedbackSent = true
                                onFeedback(true)
                            },
                    )
                    Icon(
                        Icons.Outlined.ThumbDown,
                        contentDescription = "Thumbs down",
                        tint = scheme.onSurfaceVariant,
                        modifier = Modifier
                            .size(12.dp)
                            .clickable {
                                feedbackSent = true
                                onFeedback(false)
                            },
                    )
                }
            } else {
                Text(
                    text = "感谢反馈！",
                    style = MaterialTheme.typography.labelSmall,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.padding(start = 4.dp, top = 4.dp),
                )
            }
        }
        Spacer(Modifier.width(20.dp))
    }
}

// MARK: - Status / Error

@Composable
private fun StatusLine(text: String, modifier: Modifier = Modifier) {
    Text(
        text = text,
        style = MaterialTheme.typography.bodySmall,
        color = MaterialTheme.colorScheme.onSurfaceVariant,
        textAlign = TextAlign.Center,
        modifier = modifier
            .fillMaxWidth()
            .padding(vertical = 6.dp),
    )
}

@Composable
private fun ErrorCard(text: String, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = modifier
            .padding(horizontal = 16.dp, vertical = 4.dp)
            .fillMaxWidth()
            .clip(RoundedCornerShape(12.dp))
            .background(IosColors.Red.copy(alpha = 0.08f))
            .border(0.5.dp, IosColors.Red.copy(alpha = 0.15f), RoundedCornerShape(12.dp))
            .padding(12.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalAlignment = Alignment.CenterVertically,
    ) {
        Icon(
            Icons.Filled.Warning,
            contentDescription = null,
            tint = IosColors.Red,
            modifier = Modifier.size(16.dp),
        )
        Text(
            text = text,
            style = MaterialTheme.typography.bodyLarge,
            color = IosColors.Red,
        )
    }
}

// MARK: - Evaluation / Quality / Feedback / Image

@Composable
private fun CardScaffold(modifier: Modifier, content: @Composable () -> Unit) {
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(14.dp))
            .background(MaterialTheme.colorScheme.surfaceVariant)
            .padding(horizontal = 14.dp, vertical = 12.dp),
    ) {
        content()
    }
}

@Composable
private fun EvaluationCard(eval: AppMessage.Evaluation, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    WithAvatar(eval, modifier) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Icon(Icons.Filled.Functions, contentDescription = null, tint = scheme.onSurfaceVariant, modifier = Modifier.size(11.dp))
                Text(
                    "Tool Evaluation: ${eval.tool}",
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.Medium,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    if (eval.valid) "Valid" else "Invalid",
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.SemiBold,
                    color = if (eval.valid) IosColors.Green else IosColors.Red,
                )
            }
            if (eval.issues.isNotEmpty()) {
                Column(
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(scheme.surfaceContainer)
                        .padding(10.dp),
                ) {
                    eval.issues.forEach { issue ->
                        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            Icon(Icons.Filled.Warning, contentDescription = null, tint = IosColors.Orange, modifier = Modifier.size(10.dp))
                            Text(issue, style = MaterialTheme.typography.bodySmall, color = scheme.onSurface)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun QualityCard(quality: AppMessage.Quality, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    WithAvatar(quality, modifier) {
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Icon(Icons.Filled.Star, contentDescription = null, tint = scheme.onSurfaceVariant, modifier = Modifier.size(11.dp))
                Text(
                    "Quality Assessment",
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.Medium,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    quality.score,
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.Bold,
                    color = IosColors.Orange,
                )
            }
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Text(
                    if (quality.complete) "✓ Complete" else "○ Incomplete",
                    style = MaterialTheme.typography.labelSmall,
                    color = if (quality.complete) IosColors.Green else scheme.onSurfaceVariant,
                )
                Text(
                    if (quality.referencesValid) "✓ Refs Valid" else "✗ Refs Invalid",
                    style = MaterialTheme.typography.labelSmall,
                    color = if (quality.referencesValid) IosColors.Blue else IosColors.Red,
                )
            }
            if (quality.issues.isNotEmpty()) {
                Column(
                    verticalArrangement = Arrangement.spacedBy(4.dp),
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(8.dp))
                        .background(scheme.surfaceContainer)
                        .padding(10.dp),
                ) {
                    quality.issues.forEach { issue ->
                        Row(horizontalArrangement = Arrangement.spacedBy(4.dp)) {
                            Icon(Icons.Filled.Warning, contentDescription = null, tint = IosColors.Orange, modifier = Modifier.size(10.dp))
                            Text(issue, style = MaterialTheme.typography.bodySmall, color = scheme.onSurface)
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun FeedbackCard(feedback: AppMessage.Feedback, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    WithAvatar(feedback, modifier) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Icon(
                    if (feedback.positive) Icons.Filled.ThumbUp else Icons.Filled.ThumbDown,
                    contentDescription = null,
                    tint = scheme.onSurfaceVariant,
                    modifier = Modifier.size(11.dp),
                )
                Text(
                    "Feedback",
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.Medium,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    if (feedback.positive) "Positive" else "Negative",
                    style = MaterialTheme.typography.bodySmall,
                    color = if (feedback.positive) IosColors.Green else IosColors.Red,
                )
            }
            feedback.message?.let {
                Text(it, style = MaterialTheme.typography.bodySmall, color = scheme.onSurface)
            }
        }
    }
}

@Composable
private fun ImageBubble(image: AppMessage.Image, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    WithAvatar(image, modifier) {
        Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Icon(Icons.Filled.Photo, contentDescription = null, tint = scheme.onSurfaceVariant, modifier = Modifier.size(11.dp))
                Text(
                    "Generated Image",
                    style = MaterialTheme.typography.bodySmall,
                    fontWeight = FontWeight.Medium,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    image.format.uppercase(),
                    style = MaterialTheme.typography.labelSmall,
                    color = scheme.onSurfaceVariant,
                )
            }
            SubcomposeAsyncImage(
                model = image.url,
                contentDescription = image.altText,
                contentScale = ContentScale.FillWidth,
                modifier = Modifier
                    .fillMaxWidth()
                    .heightIn(max = 300.dp)
                    .clip(RoundedCornerShape(12.dp))
                    .background(scheme.surfaceContainer),
                loading = {
                    Box(Modifier.fillMaxWidth().height(200.dp), contentAlignment = Alignment.Center) {
                        CircularProgressIndicator(modifier = Modifier.size(20.dp))
                    }
                },
                error = {
                    Box(Modifier.fillMaxWidth().height(150.dp), contentAlignment = Alignment.Center) {
                        Text("Failed to load image", style = MaterialTheme.typography.bodySmall, color = scheme.onSurfaceVariant)
                    }
                },
            )
            if (image.altText.isNotEmpty()) {
                Text(image.altText, style = MaterialTheme.typography.bodySmall, color = scheme.onSurface)
            }
            Text(
                "${image.width} × ${image.height}",
                style = MaterialTheme.typography.labelSmall,
                color = scheme.onSurfaceVariant.copy(alpha = 0.7f),
            )
        }
    }
}
