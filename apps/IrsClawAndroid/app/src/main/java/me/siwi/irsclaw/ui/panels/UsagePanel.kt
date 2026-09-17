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
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.theme.MonoStyle

private val PERIODS = listOf("today" to "今天", "7d" to "7 天", "30d" to "30 天", "all" to "全部")

/** Token/cost usage with period picker and per-session totals (iOS UsagePanel). */
@Composable
fun UsagePanel(viewModel: ClawViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var period by rememberSaveable { mutableStateOf("today") }

    androidx.compose.runtime.LaunchedEffect(period) { viewModel.fetchStats(period) }

    val stats = state.stats
    val scheme = MaterialTheme.colorScheme
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        item {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                PERIODS.forEach { (key, label) ->
                    FilterChip(
                        selected = period == key,
                        onClick = { period = key },
                        label = { Text(label) },
                    )
                }
            }
        }
        item {
            Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                StatCard("请求", (stats?.totalRequests ?: 0L).toString(), Modifier.weight(1f))
                StatCard("Tokens", formatCount(stats?.totalTokens ?: 0L), Modifier.weight(1f))
                StatCard("费用", "$%.4f".format(stats?.totalCostUsd ?: 0.0), Modifier.weight(1f))
            }
        }
        stats?.today?.let { today ->
            item {
                Column(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(12.dp))
                        .padding(12.dp),
                ) {
                    Text("今日", style = MaterialTheme.typography.labelLarge, color = scheme.onSurfaceVariant)
                    Text(
                        "${today.requests} 次请求 · ${formatCount(today.tokens.toLong())} tokens · $%.4f".format(today.costUsd),
                        style = MonoStyle,
                        color = scheme.onSurface,
                        modifier = Modifier.padding(top = 2.dp),
                    )
                }
            }
        }
        item {
            Text(
                text = "会话累计",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = scheme.onSurface,
            )
        }
        val titleById = state.sessions.associate { it.id to (it.title.ifBlank { "未命名会话" }) }
        val usages = state.sessionTokenUsage.entries.sortedByDescending { it.value.estimatedCostUsd }
        if (usages.isEmpty()) {
            item { SectionEmpty("暂无会话用量记录") }
        } else {
            items(usages, key = { it.key }) { (sessionId, usage) ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .padding(vertical = 6.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            text = titleById[sessionId] ?: sessionId.take(12),
                            style = MaterialTheme.typography.bodyMedium,
                            color = scheme.onSurface,
                            maxLines = 1,
                        )
                        Text(
                            text = usage.formattedTokens,
                            style = MonoStyle,
                            color = scheme.onSurfaceVariant,
                        )
                    }
                    usage.formattedCost?.let {
                        Text(text = it, style = MonoStyle, color = scheme.primary)
                    }
                }
            }
        }
    }
}

@Composable
private fun StatCard(label: String, value: String, modifier: Modifier = Modifier) {
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(12.dp))
            .padding(12.dp),
    ) {
        Text(text = label, style = MaterialTheme.typography.labelMedium, color = scheme.onSurfaceVariant)
        Text(
            text = value,
            style = MonoStyle,
            fontWeight = FontWeight.SemiBold,
            color = scheme.onSurface,
            modifier = Modifier.padding(top = 2.dp),
        )
    }
}

private fun formatCount(n: Long): String = when {
    n >= 1_000_000 -> "%.1fM".format(n / 1_000_000.0)
    n >= 1_000 -> "%.1fk".format(n / 1_000.0)
    else -> n.toString()
}
