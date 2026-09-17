package me.siwi.irsclaw.ui.panels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import me.siwi.irsclaw.data.model.ClawAgent
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.settings.AgentEditorSheet

/** Agents tab: profile cards + create/edit entry (iOS AgentsSettingsView subset). */
@Composable
fun AgentsPanel(viewModel: ClawViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var editing by remember { mutableStateOf<ClawAgent?>(null) }
    var showEditor by remember { mutableStateOf(false) }

    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        items(state.agents, key = { it.id }) { agent ->
            val scheme = MaterialTheme.colorScheme
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .clip(RoundedCornerShape(12.dp))
                    .padding(14.dp),
            ) {
                Text(
                    text = agent.id,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = if (agent.id == state.currentAgentId) scheme.primary else scheme.onSurface,
                )
                Text(
                    text = listOfNotNull(
                        agent.provider?.let { "provider: $it" },
                        agent.model?.let { "model: $it" },
                        "${agent.toolCount} 工具",
                    ).joinToString(" · "),
                    style = MaterialTheme.typography.bodySmall,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 2.dp),
                )
                agent.systemPrompt?.takeIf { it.isNotBlank() }?.let { prompt ->
                    Text(
                        text = prompt,
                        style = MaterialTheme.typography.bodySmall,
                        color = scheme.onSurfaceVariant,
                        maxLines = 2,
                        modifier = Modifier.padding(top = 4.dp),
                    )
                }
                TextButton(onClick = { editing = agent; showEditor = true }) {
                    Text("编辑")
                }
            }
        }
        item {
            TextButton(onClick = { editing = null; showEditor = true }) {
                Icon(Icons.Filled.Add, contentDescription = null, modifier = Modifier.padding(end = 4.dp))
                Text("新建智能体")
            }
        }
    }

    if (showEditor) {
        AgentEditorSheet(existing = editing, viewModel = viewModel, onDismiss = { showEditor = false })
    }
}
