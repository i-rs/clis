package me.siwi.irsclaw.ui.chat

import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.combinedClickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale
import me.siwi.irsclaw.data.model.SessionMeta
import me.siwi.irsclaw.ui.components.EmptyState

/**
 * Grouped session list (今天/昨天/本周/更早) with search filter — the port of the
 * iOS SessionListView. Long-press deletes. Plain column: the caller owns
 * scrolling (sidebar wraps it in a verticalScroll container).
 */
@OptIn(ExperimentalFoundationApi::class)
@Composable
fun SessionList(
    sessions: List<SessionMeta>,
    currentSessionId: String?,
    searchQuery: String,
    onSelect: (SessionMeta) -> Unit,
    onDelete: (SessionMeta) -> Unit,
    modifier: Modifier = Modifier,
) {
    val filtered = remember(sessions, searchQuery) {
        if (searchQuery.isBlank()) sessions
        else sessions.filter { it.title.contains(searchQuery, ignoreCase = true) }
    }

    if (filtered.isEmpty()) {
        EmptyState(
            icon = Icons.AutoMirrored.Filled.Chat,
            title = "没有会话",
            message = if (searchQuery.isBlank()) "开始对话后将出现在这里" else "没有匹配「$searchQuery」的会话",
            modifier = modifier,
        )
        return
    }

    Column(modifier = modifier.fillMaxWidth()) {
        groupSessions(filtered).forEach { (label, items) ->
            Text(
                text = label,
                style = MaterialTheme.typography.labelMedium,
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(horizontal = 16.dp, vertical = 6.dp),
            )
            items.forEach { session ->
                SessionRow(
                    session = session,
                    selected = session.id == currentSessionId,
                    onClick = { onSelect(session) },
                    onDelete = { onDelete(session) },
                )
            }
            HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.15f))
        }
    }
}

@OptIn(ExperimentalFoundationApi::class)
@Composable
private fun SessionRow(
    session: SessionMeta,
    selected: Boolean,
    onClick: () -> Unit,
    onDelete: () -> Unit,
) {
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .combinedClickable(onClick = onClick, onLongClick = onDelete)
            .padding(horizontal = 16.dp, vertical = 10.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Column(modifier = Modifier.weight(1f)) {
            Text(
                text = session.title.ifBlank { "未命名会话" },
                style = MaterialTheme.typography.bodyMedium,
                fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
                color = if (selected) scheme.primary else scheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(
                    text = "${session.messageCount} 条消息",
                    style = MaterialTheme.typography.labelSmall,
                    color = scheme.onSurfaceVariant,
                )
                if (session.agentId != null && session.agentId != "default") {
                    Text(
                        text = session.agentId,
                        style = MaterialTheme.typography.labelSmall,
                        color = scheme.primary,
                    )
                }
            }
        }
        Text(
            text = shortDate(session.createdAt),
            style = MaterialTheme.typography.labelSmall,
            color = scheme.onSurfaceVariant,
        )
    }
}

private fun shortDate(timestamp: Long?): String {
    if (timestamp == null) return ""
    val date = Date(timestamp * 1000)
    val cal = Calendar.getInstance().apply { time = date }
    val now = Calendar.getInstance()
    val fmt = SimpleDateFormat(
        when {
            cal.get(Calendar.YEAR) == now.get(Calendar.YEAR) &&
                cal.get(Calendar.DAY_OF_YEAR) == now.get(Calendar.DAY_OF_YEAR) -> "HH:mm"
            cal.get(Calendar.YEAR) == now.get(Calendar.YEAR) -> "MM-dd"
            else -> "yyyy-MM-dd"
        },
        Locale.getDefault(),
    )
    return fmt.format(date)
}

/** Groups sessions into 今天/昨天/本周/更早 buckets preserving recency order. */
fun groupSessions(sessions: List<SessionMeta>): List<Pair<String, List<SessionMeta>>> {
    val now = Calendar.getInstance()
    val today = Calendar.getInstance().apply {
        set(now.get(Calendar.YEAR), now.get(Calendar.MONTH), now.get(Calendar.DAY_OF_MONTH), 0, 0, 0)
        set(Calendar.MILLISECOND, 0)
    }.timeInMillis
    val yesterday = today - 24 * 3600 * 1000L
    val weekAgo = today - 7 * 24 * 3600 * 1000L

    fun bucket(s: SessionMeta): String? {
        val ts = s.createdAt ?: return null
        val ms = ts * 1000
        return when {
            ms >= today -> "今天"
            ms >= yesterday -> "昨天"
            ms >= weekAgo -> "本周"
            else -> "更早"
        }
    }

    val order = listOf("今天", "昨天", "本周", "更早")
    return order.mapNotNull { label ->
        val items = sessions.filter { bucket(it) == label }
        if (items.isEmpty()) null else label to items
    }
}
