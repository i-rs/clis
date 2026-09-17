package me.siwi.irsclaw.ui.chat

import androidx.compose.animation.animateColorAsState
import androidx.compose.animation.core.spring
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.ArrowUpward
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.OutlinedTextFieldDefaults
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.ui.components.PulsingDot
import me.siwi.irsclaw.ui.theme.IosColors

/**
 * Floating pill input bar (iOS ChatView): translucent rounded pill with shadow,
 * 38dp circular mic/send buttons, send accent swap animated with a spring.
 */
@Composable
fun InputBar(
    value: String,
    onValueChange: (String) -> Unit,
    onSend: () -> Unit,
    onStop: () -> Unit,
    onMicClick: () -> Unit,
    isProcessing: Boolean,
    modifier: Modifier = Modifier,
) {
    val scheme = MaterialTheme.colorScheme
    val hasContent = value.isNotBlank()
    val sendColor by animateColorAsState(
        targetValue = if (hasContent) scheme.primary else scheme.surfaceContainerHigh,
        animationSpec = spring(dampingRatio = 0.75f, stiffness = 380f),
        label = "send",
    )

    Row(
        modifier = modifier
            .fillMaxWidth()
            .shadow(8.dp, RoundedCornerShape(28.dp))
            .clip(RoundedCornerShape(28.dp))
            .background(scheme.surface.copy(alpha = 0.82f))
            .border(0.5.dp, Color.White.copy(alpha = 0.1f), RoundedCornerShape(28.dp))
            .padding(horizontal = 8.dp, vertical = 6.dp),
        verticalAlignment = Alignment.Bottom,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Box(
            modifier = Modifier
                .size(38.dp)
                .background(scheme.surfaceContainerHigh, CircleShape)
                .clickable(onClick = onMicClick),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                Icons.Filled.Mic,
                contentDescription = "Voice input",
                tint = scheme.onSurfaceVariant,
                modifier = Modifier.size(15.dp),
            )
        }
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.weight(1f),
            placeholder = {
                Text(
                    "Message i-rs-claw...",
                    style = MaterialTheme.typography.bodyLarge,
                    color = scheme.onSurfaceVariant.copy(alpha = 0.6f),
                )
            },
            textStyle = MaterialTheme.typography.bodyLarge,
            minLines = 1,
            maxLines = 5,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onSend = { if (!isProcessing && hasContent) onSend() }),
            colors = OutlinedTextFieldDefaults.colors(
                focusedBorderColor = Color.Transparent,
                unfocusedBorderColor = Color.Transparent,
                disabledBorderColor = Color.Transparent,
                focusedContainerColor = Color.Transparent,
                unfocusedContainerColor = Color.Transparent,
                disabledContainerColor = Color.Transparent,
            ),
        )
        Box(
            modifier = Modifier
                .size(38.dp)
                .background(sendColor, CircleShape)
                .clickable {
                    when {
                        isProcessing -> onStop()
                        hasContent -> onSend()
                    }
                },
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                Icons.Filled.ArrowUpward,
                contentDescription = if (isProcessing) "Stop" else "Send",
                tint = if (hasContent) Color.White else scheme.onSurfaceVariant,
                modifier = Modifier.size(15.dp),
            )
        }
    }
}

/** Red-tinted recording bar with pulsing dot + live transcript (iOS recording bar). */
@Composable
fun VoiceBar(
    partialText: String,
    onFinish: () -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(14.dp))
            .background(IosColors.Red.copy(alpha = 0.04f))
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        PulsingDot()
        Text(
            text = "Listening",
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.Medium,
            color = IosColors.Red.copy(alpha = 0.8f),
        )
        Text(
            text = partialText,
            style = MaterialTheme.typography.bodySmall,
            color = scheme.onSurfaceVariant,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
            modifier = Modifier.weight(1f),
        )
        TextButton(onClick = onCancel) {
            Text("取消", style = MaterialTheme.typography.bodySmall, color = scheme.onSurfaceVariant)
        }
        TextButton(onClick = onFinish) {
            Text("完成", style = MaterialTheme.typography.bodySmall, color = scheme.primary, fontWeight = FontWeight.SemiBold)
        }
    }
}
