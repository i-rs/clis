package me.siwi.irsclaw.ui.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.KeyboardActions
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Send
import androidx.compose.material.icons.filled.Mic
import androidx.compose.material.icons.filled.Stop
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.ui.components.PulsingDot

private val ColorTransparent = Color.Transparent

/**
 * Floating pill input bar (mic + auto-growing field + animated send) — the port
 * of the iOS ChatView input. While [isProcessing], send becomes a stop button.
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
    Row(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(28.dp))
            .background(scheme.surfaceContainerHigh)
            .padding(horizontal = 6.dp, vertical = 4.dp),
        verticalAlignment = Alignment.Bottom,
    ) {
        IconButton(onClick = onMicClick) {
            Icon(
                Icons.Filled.Mic,
                contentDescription = "语音输入",
                tint = scheme.primary,
            )
        }
        OutlinedTextField(
            value = value,
            onValueChange = onValueChange,
            modifier = Modifier.weight(1f),
            placeholder = { Text("发送消息…", style = MaterialTheme.typography.bodyMedium) },
            textStyle = MaterialTheme.typography.bodyLarge,
            minLines = 1,
            maxLines = 5,
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Send),
            keyboardActions = KeyboardActions(onSend = { if (!isProcessing && value.isNotBlank()) onSend() }),
            colors = androidx.compose.material3.OutlinedTextFieldDefaults.colors(
                focusedBorderColor = ColorTransparent,
                unfocusedBorderColor = ColorTransparent,
                disabledBorderColor = ColorTransparent,
                focusedContainerColor = ColorTransparent,
                unfocusedContainerColor = ColorTransparent,
                disabledContainerColor = ColorTransparent,
            ),
        )
        IconButton(
            onClick = { if (isProcessing) onStop() else if (value.isNotBlank()) onSend() },
        ) {
            Icon(
                imageVector = if (isProcessing) Icons.Filled.Stop else Icons.AutoMirrored.Filled.Send,
                contentDescription = if (isProcessing) "停止" else "发送",
                tint = if (isProcessing || value.isNotBlank()) scheme.primary else scheme.onSurfaceVariant,
            )
        }
    }
}

/** Full-width recording bar with pulsing dot + live transcript (iOS recording bar). */
@Composable
fun VoiceBar(
    partialText: String,
    onFinish: () -> Unit,
    onCancel: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = modifier
            .fillMaxWidth()
            .clip(RoundedCornerShape(28.dp))
            .background(scheme.primaryContainer)
            .padding(horizontal = 16.dp, vertical = 12.dp),
    ) {
        Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(10.dp)) {
            PulsingDot()
            Text(
                text = "正在聆听…",
                style = MaterialTheme.typography.labelLarge,
                color = scheme.onPrimaryContainer,
            )
            Text(
                text = partialText,
                style = MaterialTheme.typography.bodyMedium,
                color = scheme.onPrimaryContainer.copy(alpha = 0.75f),
                maxLines = 2,
                modifier = Modifier.weight(1f),
            )
            TextButton(onClick = onCancel) { Text("取消", color = scheme.onPrimaryContainer) }
            TextButton(onClick = onFinish) { Text("完成", color = scheme.primary) }
        }
    }
}
