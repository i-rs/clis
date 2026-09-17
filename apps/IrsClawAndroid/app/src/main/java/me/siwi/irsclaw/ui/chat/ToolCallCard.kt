package me.siwi.irsclaw.ui.chat

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.CheckCircle
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Refresh
import androidx.compose.material.icons.outlined.Error
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
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.data.model.AppMessage
import me.siwi.irsclaw.ui.components.rememberJsonHighlight
import me.siwi.irsclaw.ui.theme.MonoStyle

private val SuccessGreen = Color(0xFF30D158)

/**
 * Expandable tool-call card: collapsed shows a one-line result preview; expanded
 * shows Arguments/Result blocks with JSON syntax highlighting (iOS ToolCallCard).
 */
@Composable
fun ToolCallCard(toolCall: AppMessage.ToolCall) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    val scheme = MaterialTheme.colorScheme
    val chevronRotation by animateFloatAsState(if (expanded) 180f else 0f, label = "chevron")

    val status: Pair<ImageVector, Color> = when {
        toolCall.result.contains("error", true) || toolCall.result.contains("failed", true) ->
            Icons.Outlined.Error to scheme.error
        toolCall.result.isEmpty() ->
            Icons.Filled.Refresh to scheme.onSurfaceVariant
        else ->
            Icons.Filled.CheckCircle to SuccessGreen
    }

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .animateContentSize()
            .background(scheme.surfaceContainer, RoundedCornerShape(12.dp))
            .clickable { expanded = !expanded }
            .padding(12.dp),
    ) {
        Row(
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(status.first, contentDescription = null, tint = status.second, modifier = Modifier.size(16.dp))
            Text(
                text = toolCall.name,
                style = MaterialTheme.typography.labelLarge,
                fontWeight = FontWeight.SemiBold,
                color = scheme.onSurface,
            )
            if (toolCall.totalSteps > 0) {
                Text(
                    text = "${toolCall.step}/${toolCall.totalSteps}",
                    style = MonoStyle,
                    color = scheme.onSurfaceVariant,
                )
            }
            Spacer(Modifier.weight(1f))
            Icon(
                Icons.Filled.KeyboardArrowDown,
                contentDescription = if (expanded) "收起" else "展开",
                tint = scheme.onSurfaceVariant,
                modifier = Modifier
                    .size(18.dp)
                    .graphicsLayer { rotationZ = chevronRotation },
            )
        }

        if (!expanded && toolCall.result.isNotEmpty()) {
            Text(
                text = toolCall.result.take(50) + if (toolCall.result.length > 50) "…" else "",
                style = MonoStyle,
                color = scheme.onSurfaceVariant,
                maxLines = 1,
                modifier = Modifier.padding(top = 6.dp),
            )
        }

        AnimatedVisibility(visible = expanded) {
            Column(
                modifier = Modifier.padding(top = 10.dp),
                verticalArrangement = Arrangement.spacedBy(10.dp),
            ) {
                JsonBlock(title = "Arguments", content = toolCall.args.ifBlank { "{}" })
                JsonBlock(title = "Result", content = toolCall.result.ifBlank { "(无输出)" })
            }
        }
    }
}

@Composable
private fun JsonBlock(title: String, content: String) {
    val scheme = MaterialTheme.colorScheme
    val highlighted: AnnotatedString = rememberJsonHighlight(content)
    Column(
        modifier = Modifier
            .fillMaxWidth()
            .background(scheme.surfaceContainerHigh, RoundedCornerShape(8.dp))
            .padding(10.dp),
    ) {
        Text(
            text = title,
            style = MaterialTheme.typography.labelMedium,
            fontWeight = FontWeight.SemiBold,
            color = scheme.onSurfaceVariant,
        )
        androidx.compose.foundation.text.selection.SelectionContainer {
            Text(
                text = highlighted,
                style = MonoStyle,
                color = scheme.onSurface,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 4.dp),
            )
        }
    }
}
