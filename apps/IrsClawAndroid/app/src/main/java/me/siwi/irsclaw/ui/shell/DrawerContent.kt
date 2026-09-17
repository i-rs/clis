package me.siwi.irsclaw.ui.shell

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.horizontalScroll
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material.icons.filled.Book
import androidx.compose.material.icons.filled.InsertChart
import androidx.compose.material.icons.filled.Person2
import androidx.compose.material.icons.filled.Psychology
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.Widgets
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import me.siwi.irsclaw.data.model.SessionMeta
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.components.AgentChip
import me.siwi.irsclaw.ui.panels.SidebarTab
import me.siwi.irsclaw.ui.theme.IosColors

private data class NavEntry(val tab: SidebarTab, val icon: ImageVector, val color: Color)

private val NAV_ENTRIES = listOf(
    NavEntry(SidebarTab.SESSIONS, Icons.AutoMirrored.Filled.Chat, IosColors.Blue),
    NavEntry(SidebarTab.TOOLS, Icons.Filled.Widgets, IosColors.Orange),
    NavEntry(SidebarTab.SKILLS, Icons.Filled.Book, IosColors.Green),
    NavEntry(SidebarTab.PLUGINS, Icons.Filled.Psychology, IosColors.Purple),
    NavEntry(SidebarTab.USAGE, Icons.Filled.InsertChart, IosColors.Blue),
    NavEntry(SidebarTab.AGENTS, Icons.Filled.Person2, IosColors.Teal),
)

/**
 * Phone drawer content (iOS DrawerMenuView): "Agent" card, "Browse" 2-column
 * grid with tinted icon tiles, and a "Recent Sessions" grouped card.
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
    val cardColor = scheme.surfaceContainerHigh // secondarySystemGroupedBackground: #FFF / #2C2C2E

    Column(
        modifier = modifier
            .fillMaxWidth()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(24.dp),
    ) {
        // Agent card
        Column(
            modifier = Modifier
                .fillMaxWidth()
                .background(cardColor, RoundedCornerShape(16.dp))
                .padding(16.dp),
            verticalArrangement = Arrangement.spacedBy(12.dp),
        ) {
            Text(
                "Agent",
                style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                fontWeight = FontWeight.SemiBold,
                color = scheme.onSurfaceVariant,
            )
            if (state.agents.size <= 1) {
                val current = state.agents.firstOrNull { it.id == state.currentAgentId }
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
                    Box(
                        modifier = Modifier
                            .size(36.dp)
                            .background(Brush.linearGradient(listOf(IosColors.Blue, IosColors.Purple)), CircleShape),
                        contentAlignment = Alignment.Center,
                    ) {
                        Icon(Icons.Filled.Star, contentDescription = null, tint = Color.White, modifier = Modifier.size(14.dp))
                    }
                    Text(
                        text = state.currentAgentId,
                        style = MaterialTheme.typography.bodyLarge,
                        fontWeight = FontWeight.Medium,
                        color = scheme.onSurface,
                    )
                }
                current?.let {
                    Text(
                        text = listOfNotNull(it.provider, it.model).joinToString(" · ").ifBlank { "Default configuration" },
                        style = MaterialTheme.typography.bodySmall,
                        color = scheme.onSurfaceVariant,
                    )
                }
            } else {
                Row(
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                    modifier = Modifier.horizontalScroll(rememberScrollState()),
                ) {
                    state.agents.forEach { agent ->
                        AgentChip(
                            agentId = agent.id,
                            selected = agent.id == state.currentAgentId,
                            onClick = { viewModel.switchAgent(agent.id) },
                        )
                    }
                }
            }
        }

        // Browse grid
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(
                "Browse",
                style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                fontWeight = FontWeight.SemiBold,
                color = scheme.onSurfaceVariant,
            )
            NAV_ENTRIES.chunked(2).forEach { rowEntries ->
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    rowEntries.forEach { entry ->
                        NavGridCell(entry, cardColor, Modifier.weight(1f)) { onNavigate(entry.tab) }
                    }
                    if (rowEntries.size == 1) Spacer(Modifier.weight(1f))
                }
            }
        }

        // Recent sessions
        Column(verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(
                    "Recent Sessions",
                    style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                    fontWeight = FontWeight.SemiBold,
                    color = scheme.onSurfaceVariant,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    "See All",
                    style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                    color = scheme.primary,
                    modifier = Modifier.clickable { onNavigate(SidebarTab.SESSIONS) },
                )
            }
            Column(
                modifier = Modifier
                    .fillMaxWidth()
                    .background(cardColor, RoundedCornerShape(16.dp)),
            ) {
                state.sessions.take(5).forEachIndexed { index, session ->
                    Row(
                        modifier = Modifier
                            .fillMaxWidth()
                            .clickable { onSelectSession(session) }
                            .padding(16.dp),
                        horizontalArrangement = Arrangement.spacedBy(12.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Icon(
                            Icons.AutoMirrored.Filled.Chat,
                            contentDescription = null,
                            tint = scheme.onSurfaceVariant,
                            modifier = Modifier.size(14.dp),
                        )
                        Text(
                            text = session.title.ifBlank { "Untitled" },
                            style = MaterialTheme.typography.bodyLarge,
                            color = scheme.onSurface,
                            maxLines = 1,
                            overflow = TextOverflow.Ellipsis,
                            modifier = Modifier.weight(1f),
                        )
                        Text(
                            text = recentDate(session),
                            style = MaterialTheme.typography.bodySmall,
                            color = scheme.onSurfaceVariant.copy(alpha = 0.7f),
                        )
                    }
                    if (index < state.sessions.take(5).lastIndex) {
                        HorizontalDivider(modifier = Modifier.padding(start = 56.dp))
                    }
                }
            }
        }
    }
}

@Composable
private fun NavGridCell(entry: NavEntry, cardColor: Color, modifier: Modifier = Modifier, onClick: () -> Unit) {
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = modifier
            .background(cardColor, RoundedCornerShape(16.dp))
            .clickable(onClick = onClick)
            .padding(vertical = 16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Box(
            modifier = Modifier
                .size(48.dp)
                .background(entry.color.copy(alpha = 0.15f), RoundedCornerShape(12.dp)),
            contentAlignment = Alignment.Center,
        ) {
            Icon(entry.icon, contentDescription = null, tint = entry.color, modifier = Modifier.size(20.dp))
        }
        Text(
            text = entry.tab.label,
            style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
            fontWeight = FontWeight.Medium,
            color = scheme.onSurface,
        )
    }
}

private fun recentDate(session: SessionMeta): String {
    val ts = session.createdAt ?: return ""
    val cal = java.util.Calendar.getInstance().apply { timeInMillis = ts * 1000 }
    val now = java.util.Calendar.getInstance()
    val fmt = java.text.SimpleDateFormat(
        when {
            cal.get(java.util.Calendar.YEAR) == now.get(java.util.Calendar.YEAR) &&
                cal.get(java.util.Calendar.DAY_OF_YEAR) == now.get(java.util.Calendar.DAY_OF_YEAR) -> "HH:mm"
            else -> "MM-dd"
        },
        java.util.Locale.getDefault(),
    )
    return fmt.format(cal.time)
}
