package me.siwi.irsclaw.ui.chat

import android.Manifest
import android.content.pm.PackageManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material.icons.filled.Add
import androidx.compose.material.icons.filled.Menu
import androidx.compose.material.icons.filled.Settings
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import me.siwi.irsclaw.data.model.AppMessage
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.logic.rememberVoiceInput
import me.siwi.irsclaw.ui.components.AgentChip
import me.siwi.irsclaw.ui.components.EmptyState

/**
 * The chat detail: top bar (menu / agent chips / new chat / settings), message
 * list with auto-scroll, error banner, and the voice-aware input area.
 */
@Composable
fun ChatScreen(
    viewModel: ClawViewModel,
    onMenuClick: (() -> Unit)?,
    onOpenSettings: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val settings by viewModel.settings.collectAsStateWithLifecycle()
    val listState = rememberLazyListState()
    val context = LocalContext.current

    var input by rememberSaveable { mutableStateOf("") }

    val voice = rememberVoiceInput { recognized ->
        input = (if (input.isBlank()) "" else "$input ") + recognized
    }

    var micGranted by rememberSaveable {
        mutableStateOf(
            ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED,
        )
    }
    val micPermission = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        micGranted = granted
        if (granted) voice.start()
    }

    fun startVoice() {
        if (micGranted) voice.start() else micPermission.launch(Manifest.permission.RECORD_AUDIO)
    }

    // Auto-scroll on every streaming mutation (iOS messageVersion → scroll to bottom).
    LaunchedEffect(state.messageVersion) {
        val count = state.messages.size
        if (count > 0) {
            if (state.isProcessing) listState.scrollToItem(count - 1) else listState.animateScrollToItem(count - 1)
        }
    }

    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        Column {
            // Top bar
            Row(
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 8.dp, vertical = 4.dp),
                verticalAlignment = Alignment.CenterVertically,
                horizontalArrangement = Arrangement.spacedBy(4.dp),
            ) {
                if (onMenuClick != null) {
                    IconButton(onClick = onMenuClick) {
                        Icon(Icons.Filled.Menu, contentDescription = "菜单")
                    }
                }
                Row(
                    modifier = Modifier
                        .weight(1f)
                        .padding(horizontal = 4.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    (listOf("default") + state.agents.map { it.id }).distinct().take(3).forEach { agentId ->
                        AgentChip(
                            agentId = agentId,
                            selected = agentId == state.currentAgentId,
                            onClick = { viewModel.switchAgent(agentId) },
                        )
                    }
                }
                IconButton(onClick = { viewModel.createSession() }) {
                    Icon(Icons.Filled.Add, contentDescription = "新会话")
                }
                IconButton(onClick = onOpenSettings) {
                    Icon(Icons.Filled.Settings, contentDescription = "设置")
                }
            }

            // Error banner
            state.errorMessage?.let { message ->
                Surface(
                    color = MaterialTheme.colorScheme.error.copy(alpha = 0.12f),
                    modifier = Modifier.fillMaxWidth(),
                ) {
                    Row(
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 8.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Text(
                            text = message,
                            style = MaterialTheme.typography.bodySmall,
                            color = MaterialTheme.colorScheme.error,
                            modifier = Modifier.weight(1f),
                        )
                        Text(
                            text = "关闭",
                            style = MaterialTheme.typography.labelMedium,
                            color = MaterialTheme.colorScheme.error,
                            modifier = Modifier
                                .padding(start = 8.dp)
                                .clickable { viewModel.clearError() },
                        )
                    }
                }
            }

            // Messages
            Box(modifier = Modifier.weight(1f)) {
                if (state.messages.isEmpty()) {
                    EmptyState(
                        icon = Icons.AutoMirrored.Filled.Chat,
                        title = "开始新的对话",
                        message = "向 AI 助理发送第一条消息",
                        modifier = Modifier.align(Alignment.Center),
                    )
                } else {
                    LazyColumn(
                        state = listState,
                        modifier = Modifier.fillMaxSize(),
                        contentPadding = PaddingValues(vertical = 8.dp),
                    ) {
                        items(state.messages, key = { it.id }) { item ->
                            MessageBubble(
                                item = item,
                                onFeedback = { positive ->
                                    state.currentSession?.let { session ->
                                        viewModel.postFeedback(session.id, positive)
                                    }
                                },
                            )
                        }
                    }
                }
            }

            // Input area: recording bar replaces the input while listening.
            Box(Modifier.padding(horizontal = 12.dp, vertical = 10.dp)) {
                if (voice.isListening) {
                    VoiceBar(
                        partialText = voice.partialText,
                        onFinish = { voice.finish() },
                        onCancel = { voice.cancel() },
                    )
                } else {
                    InputBar(
                        value = input,
                        onValueChange = { input = it },
                        onSend = {
                            viewModel.sendMessage(input.trim())
                            input = ""
                        },
                        onStop = { viewModel.stopStreaming() },
                        onMicClick = ::startVoice,
                        isProcessing = state.isProcessing,
                    )
                }
                voice.error?.let { err ->
                    Text(
                        text = err,
                        style = MaterialTheme.typography.labelMedium,
                        color = MaterialTheme.colorScheme.error,
                        textAlign = TextAlign.Center,
                        modifier = Modifier
                            .align(Alignment.TopCenter)
                            .padding(bottom = 68.dp),
                    )
                }
            }
        }
    }
}
