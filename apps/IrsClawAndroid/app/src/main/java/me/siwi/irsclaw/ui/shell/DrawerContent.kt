package me.siwi.irsclaw.ui.shell

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
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material.icons.filled.Bolt
import androidx.compose.material.icons.filled.Extension
import androidx.compose.material.icons.filled.InsertChart
import androidx.compose.material.icons.filled.SmartToy
import androidx.compose.material.icons.filled.Widgets
import androidx.compose.material.icons.outlined.AccountTree
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import me.siwi.irsclaw.data.model.SessionMeta
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.components.AgentChip
import me.siwi.irsclaw.ui.panels.SidebarTab

/**
 * Phone drawer content: agent card + 2-column nav grid + recent sessions —
 * the port of the iOS DrawerMenuView.
 */
@Composable
fun DrawerContent(
    viewModel: ClawViewModel,
    onNavigate: (SidebarTab) -> Unit,
    onSelectSession: (SessionMeta) -> Unit,
    modifier: Modifier = Modifier,
) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    val scheme = MaterialTheme.colorScheme

    Column(modifier = modifier.fillMaxWidth().padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
        // Agent card
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(scheme.primaryContainer.copy(alpha = 0.6f), RoundedCornerShape(14.dp))
                .padding(14.dp),
        ) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Icon(Icons.Filled.Bolt, contentDescription = null, tint = scheme.primary)
                Text(
                    text = state.currentAgentId,
                    style = MaterialTheme.typography.titleMedium,
                    fontWeight = FontWeight.SemiBold,
                    color = scheme.onSurface,
                )
            }
            val current = state.agents.firstOrNull { it.id == state.currentAgentId }
            current?.let {
                Text(
                    text = listOfNotNull(it.provider, it.model).joinToString(" · ").ifBlank { "默认配置" },
                    style = MaterialTheme.typography.bodySmall,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 2.dp),
                )
            }
            Row(
                horizontalArrangement = Arrangement.spacedBy(6.dp),
                modifier = Modifier.padding(top = 10.dp),
            ) {
                state.agents.take(4).forEach { agent ->
                    AgentChip(
                        agentId = agent.id,
                        selected = agent.id == state.currentAgentId,
                        onClick = { viewModel.switchAgent(agent.id) },
                    )
                }
            }
        }

        // 2-column nav grid
        val entries = listOf(
            SidebarTab.SESSIONS to Icons.AutoMirrored.Filled.Chat,
            SidebarTab.TOOLS to Icons.Filled.Widgets,
            SidebarTab.SKILLS to Icons.Filled.Extension,
            SidebarTab.PLUGINS to Icons.Outlined.AccountTree,
            SidebarTab.USAGE to Icons.Filled.InsertChart,
            SidebarTab.AGENTS to Icons.Filled.SmartToy,
        )
        Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
            entries.chunked(2).forEach { rowEntries ->
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    rowEntries.forEach { (tab, icon) ->
                        NavGridCell(tab, icon, Modifier.weight(1f)) { onNavigate(tab) }
                    }
                    if (rowEntries.size == 1) Spacer(Modifier.weight(1f))
                }
            }
        }

        // Recent sessions
        Text(
            text = "最近会话",
            style = MaterialTheme.typography.labelLarge,
            color = scheme.onSurfaceVariant,
            modifier = Modifier.padding(top = 4.dp),
        )
        state.sessions.take(5).forEach { session ->
            Text(
                text = session.title.ifBlank { "未命名会话" },
                style = MaterialTheme.typography.bodyMedium,
                color = scheme.onSurface,
                maxLines = 1,
                modifier = Modifier
                    .fillMaxWidth()
                    .clickable { onSelectSession(session) }
                    .padding(vertical = 6.dp),
            )
        }
    }
}

@Composable
private fun NavGridCell(tab: SidebarTab, icon: ImageVector, modifier: Modifier = Modifier, onClick: () -> Unit) {
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = modifier
            .background(scheme.surfaceContainerHigh, RoundedCornerShape(12.dp))
            .clickable(onClick = onClick)
            .padding(vertical = 12.dp, horizontal = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(8.dp),
    ) {
        Icon(icon, contentDescription = null, tint = scheme.primary, modifier = Modifier.size(18.dp))
        Text(tab.label, style = MaterialTheme.typography.labelLarge, color = scheme.onSurface)
    }
}
