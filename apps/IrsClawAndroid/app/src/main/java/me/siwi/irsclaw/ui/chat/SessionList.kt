package me.siwi.irsclaw.ui.chat

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Chat
import androidx.compose.material.icons.filled.ChevronRight
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SwipeToDismissBox
import androidx.compose.material3.SwipeToDismissBoxValue
import androidx.compose.material3.Text
import androidx.compose.material3.rememberSwipeToDismissBoxState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import java.text.SimpleDateFormat
import java.util.Calendar
import java.util.Date
import java.util.Locale
import me.siwi.irsclaw.data.model.SessionMeta
import me.siwi.irsclaw.ui.components.EmptyState

/**
 * Grouped session list (Today/Yesterday/This Week/Earlier) with search filter —
 * the port of the iOS SessionListView: plain rows with a 44dp leading circle,
 * trailing date + chevron. The caller owns scrolling; swipe deletes.
 */
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
            title = "No Sessions",
            message = if (searchQuery.isBlank()) "Start a conversation and it will appear here" else "No sessions matching \"$searchQuery\"",
            modifier = modifier,
        )
        return
    }

    Column(modifier = modifier.fillMaxWidth()) {
        groupSessions(filtered).forEach { (label, items) ->
            Text(
                text = label,
                style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                fontWeight = FontWeight.SemiBold,
                color = MaterialTheme.colorScheme.onSurfaceVariant,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(start = 20.dp, top = 16.dp, bottom = 4.dp),
            )
            items.forEach { session ->
                SwipeDeleteRow(onDelete = { onDelete(session) }) {
                    SessionRow(
                        session = session,
                        selected = session.id == currentSessionId,
                        onClick = { onSelect(session) },
                    )
                }
            }
        }
    }
}

@OptIn(ExperimentalMaterial3Api::class)
@Composable
private fun SwipeDeleteRow(onDelete: () -> Unit, content: @Composable () -> Unit) {
    val dismissState = rememberSwipeToDismissBoxState(
        confirmValueChange = { value ->
            if (value == SwipeToDismissBoxValue.EndToStart) {
                onDelete()
                true
            } else {
                false
            }
        },
    )
    SwipeToDismissBox(
        state = dismissState,
        enableDismissFromStartToEnd = false,
        backgroundContent = {
            Box(
                Modifier
                    .fillMaxWidth()
                    .background(MaterialTheme.colorScheme.error)
                    .padding(end = 24.dp),
                contentAlignment = Alignment.CenterEnd,
            ) {
                Icon(
                    Icons.Filled.Delete,
                    contentDescription = "Delete",
                    tint = MaterialTheme.colorScheme.onError,
                )
            }
        },
    ) {
        content()
    }
}

@Composable
private fun SessionRow(session: SessionMeta, selected: Boolean, onClick: () -> Unit) {
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .padding(horizontal = 20.dp, vertical = 12.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Box(
            modifier = Modifier
                .size(44.dp)
                .background(
                    if (selected) scheme.primary.copy(alpha = 0.15f) else scheme.onSurfaceVariant.copy(alpha = 0.1f),
                    CircleShape,
                ),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                Icons.AutoMirrored.Filled.Chat,
                contentDescription = null,
                tint = if (selected) scheme.primary else scheme.onSurfaceVariant,
                modifier = Modifier.size(18.dp),
            )
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(
                text = session.title.ifBlank { "Untitled" },
                style = MaterialTheme.typography.bodyLarge,
                fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Medium,
                color = scheme.onSurface,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                Text(
                    text = "${session.messageCount} messages",
                    style = MaterialTheme.typography.labelSmall,
                    color = scheme.onSurfaceVariant.copy(alpha = 0.7f),
                )
                if (session.agentId != null && session.agentId != "default") {
                    Text(
                        text = session.agentId,
                        style = MaterialTheme.typography.labelSmall,
                        fontWeight = FontWeight.Medium,
                        color = scheme.primary,
                        modifier = Modifier
                            .background(scheme.primary.copy(alpha = 0.12f), RoundedCornerShape(50))
                            .padding(horizontal = 6.dp, vertical = 2.dp),
                    )
                }
            }
        }
        Text(
            text = shortDate(session.createdAt),
            style = MaterialTheme.typography.labelSmall,
            color = scheme.onSurfaceVariant.copy(alpha = 0.7f),
        )
        Icon(
            Icons.Filled.ChevronRight,
            contentDescription = null,
            tint = scheme.onSurfaceVariant.copy(alpha = 0.5f),
            modifier = Modifier.size(12.dp),
        )
    }
}

private fun shortDate(timestamp: Long?): String {
    if (timestamp == null) return ""
    val date = Date(timestamp * 1000)
    val cal = Calendar.getInstance().apply { time = date }
    val now = Calendar.getInstance()
    return when {
        cal.get(Calendar.YEAR) == now.get(Calendar.YEAR) &&
            cal.get(Calendar.DAY_OF_YEAR) == now.get(Calendar.DAY_OF_YEAR) ->
            SimpleDateFormat("HH:mm", Locale.getDefault()).format(date)
        cal.get(Calendar.YEAR) == now.get(Calendar.YEAR) &&
            cal.get(Calendar.DAY_OF_YEAR) == now.get(Calendar.DAY_OF_YEAR) - 1 -> "Yesterday"
        else -> SimpleDateFormat("MM-dd", Locale.getDefault()).format(date)
    }
}

/** Groups sessions into Today/Yesterday/This Week/Earlier buckets, recency order. */
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
            ms >= today -> "Today"
            ms >= yesterday -> "Yesterday"
            ms >= weekAgo -> "This Week"
            else -> "Earlier"
        }
    }

    val order = listOf("Today", "Yesterday", "This Week", "Earlier")
    return order.mapNotNull { label ->
        val items = sessions.filter { bucket(it) == label }
        if (items.isEmpty()) null else label to items
    }
}
