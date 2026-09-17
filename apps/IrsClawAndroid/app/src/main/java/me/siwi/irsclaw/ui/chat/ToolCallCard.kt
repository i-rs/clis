package me.siwi.irsclaw.ui.chat

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.spring
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowRightAlt
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Notes
import androidx.compose.material.icons.filled.Cancel
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
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.data.model.AppMessage
import me.siwi.irsclaw.ui.components.rememberJsonHighlight
import me.siwi.irsclaw.ui.theme.IosColors
import me.siwi.irsclaw.ui.theme.MonoStyle

private enum class ToolStatus { IN_PROGRESS, SUCCESS, FAILURE }

private fun statusOf(result: String): ToolStatus = when {
    result.isEmpty() -> ToolStatus.IN_PROGRESS
    listOf("error", "failed", "failure", "panic").any { result.contains(it, true) } -> ToolStatus.FAILURE
    else -> ToolStatus.SUCCESS
}

/**
 * Expandable tool-call card (iOS ToolCallCard): secondary-background card with a
 * hairline border, 24dp tinted status badge, 50-char collapsed result preview,
 * and Arguments/Result blocks with JSON highlighting.
 */
@Composable
fun ToolCallCard(toolCall: AppMessage.ToolCall) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    val scheme = MaterialTheme.colorScheme
    val status = statusOf(toolCall.result)

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .animateContentSize(spring(dampingRatio = 0.8f, stiffness = 380f))
            .background(scheme.surfaceVariant, RoundedCornerShape(14.dp))
            .border(1.dp, Color(0xFF8E8E93).copy(alpha = 0.15f), RoundedCornerShape(14.dp))
            .clickable { expanded = !expanded },
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(10.dp),
        ) {
            StatusBadge(status)
            Text(
                text = toolCall.name,
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = FontWeight.Medium,
                color = scheme.onSurface,
            )
            Spacer(Modifier.weight(1f))
            if (!expanded && toolCall.result.isNotEmpty()) {
                Text(
                    text = toolCall.result.take(50) + if (toolCall.result.length > 50) "…" else "",
                    style = MaterialTheme.typography.bodySmall,
                    color = scheme.onSurfaceVariant.copy(alpha = 0.7f),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                    modifier = Modifier.widthIn(max = 160.dp),
                )
            }
            Icon(
                if (expanded) Icons.Filled.KeyboardArrowDown else Icons.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = scheme.onSurfaceVariant.copy(alpha = 0.7f),
                modifier = Modifier
                    .size(14.dp)
                    .width(12.dp),
            )
        }

        AnimatedVisibility(visible = expanded) {
            Column {
                androidx.compose.material3.HorizontalDivider(modifier = Modifier.padding(horizontal = 12.dp))
                Column(
                    modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp),
                    verticalArrangement = Arrangement.spacedBy(12.dp),
                ) {
                    JsonBlock(
                        icon = Icons.Filled.ArrowRightAlt,
                        title = "Arguments",
                        content = toolCall.args.ifBlank { "{}" },
                        contentBackground = scheme.surfaceContainerHigh,
                    )
                    JsonBlock(
                        icon = Icons.Filled.Notes,
                        title = "Result",
                        content = toolCall.result.ifBlank { "(empty)" },
                        contentBackground = when (status) {
                            ToolStatus.FAILURE -> IosColors.Red.copy(alpha = 0.05f)
                            ToolStatus.IN_PROGRESS -> IosColors.Orange.copy(alpha = 0.05f)
                            ToolStatus.SUCCESS -> IosColors.Green.copy(alpha = 0.05f)
                        },
                    )
                }
            }
        }
    }
}

@Composable
private fun StatusBadge(status: ToolStatus) {
    val scheme = MaterialTheme.colorScheme
    val tint = when (status) {
        ToolStatus.IN_PROGRESS -> IosColors.Orange.copy(alpha = 0.7f)
        ToolStatus.SUCCESS -> IosColors.Green.copy(alpha = 0.75f)
        ToolStatus.FAILURE -> IosColors.Red.copy(alpha = 0.75f)
    }
    val icon: ImageVector = when (status) {
        ToolStatus.SUCCESS -> Icons.Filled.CheckCircle
        ToolStatus.FAILURE -> Icons.Filled.Cancel
        ToolStatus.IN_PROGRESS -> Icons.Filled.CheckCircle // placeholder, spinner drawn instead
    }
    Box(
        modifier = Modifier
            .size(24.dp)
            .background(tint.copy(alpha = 0.15f), CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        if (status == ToolStatus.IN_PROGRESS) {
            CircularProgressIndicator(color = IosColors.Orange.copy(alpha = 0.7f), strokeWidth = 1.5.dp, modifier = Modifier.size(13.dp))
        } else {
            Icon(icon, contentDescription = null, tint = tint, modifier = Modifier.size(14.dp))
        }
    }
}

@Composable
private fun JsonBlock(icon: ImageVector, title: String, content: String, contentBackground: Color) {
    val scheme = MaterialTheme.colorScheme
    val highlighted: AnnotatedString = rememberJsonHighlight(content)
    Column(verticalArrangement = Arrangement.spacedBy(5.dp)) {
        Row(
            horizontalArrangement = Arrangement.spacedBy(5.dp),
            verticalAlignment = Alignment.CenterVertically,
        ) {
            Icon(icon, contentDescription = null, tint = scheme.onSurface.copy(alpha = 0.5f), modifier = Modifier.size(11.dp))
            Text(
                text = title,
                style = MaterialTheme.typography.bodySmall,
                fontWeight = FontWeight.SemiBold,
                color = scheme.onSurface,
            )
        }
        androidx.compose.foundation.text.selection.SelectionContainer {
            Text(
                text = highlighted,
                style = MonoStyle,
                color = scheme.onSurface,
                modifier = Modifier
                    .fillMaxWidth()
                    .background(contentBackground, RoundedCornerShape(8.dp))
                    .padding(10.dp),
            )
        }
    }
}
