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
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SegmentedButton
import androidx.compose.material3.SegmentedButtonDefaults
import androidx.compose.material3.SingleChoiceSegmentedButtonRow
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.theme.IosColors
import me.siwi.irsclaw.ui.theme.MonoStyle

private val PERIODS = listOf("today" to "Today", "7d" to "7 Days", "30d" to "30 Days", "all" to "All")

/** Token/cost usage (iOS UsagePanel): segmented period picker, colored stat trio, per-session totals. */
@Composable
fun UsagePanel(viewModel: ClawViewModel, modifier: Modifier = Modifier) {
    val state by viewModel.state.collectAsStateWithLifecycle()
    var period by rememberSaveable { mutableStateOf("today") }

    androidx.compose.runtime.LaunchedEffect(period) { viewModel.fetchStats(period) }

    val stats = state.stats
    val scheme = MaterialTheme.colorScheme
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(20.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        item {
            Text(
                "Total Tokens",
                style = MaterialTheme.typography.headlineMedium.copy(fontSize = 22.sp),
                fontWeight = FontWeight.SemiBold,
                color = scheme.onSurface,
            )
        }
        if (stats != null) {
            item {
                Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    Row(horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                        StatItem("Requests", formatCount(stats.totalRequests), IosColors.Blue, Modifier.weight(1f))
                        StatItem("Tokens", formatCount(stats.totalTokens), IosColors.Green, Modifier.weight(1f))
                        StatItem("Cost", "$%.4f".format(stats.totalCostUsd), IosColors.Orange, Modifier.weight(1f))
                    }
                    stats.today?.let { today ->
                        Text(
                            text = "Today: ${today.requests} requests · ${formatCount(today.tokens.toLong())} tokens · $%.4f".format(today.costUsd),
                            style = MonoStyle,
                            color = scheme.onSurfaceVariant,
                        )
                    }
                }
            }
        }
        item {
            SingleChoiceSegmentedButtonRow(Modifier.fillMaxWidth()) {
                PERIODS.forEachIndexed { index, (key, label) ->
                    SegmentedButton(
                        selected = period == key,
                        onClick = { period = key },
                        shape = SegmentedButtonDefaults.itemShape(index = index, count = PERIODS.size),
                    ) {
                        Text(label, style = MaterialTheme.typography.labelMedium)
                    }
                }
            }
        }
        item {
            Text(
                "Sessions",
                style = MaterialTheme.typography.titleMedium,
                fontWeight = FontWeight.SemiBold,
                color = scheme.onSurface,
            )
        }
        val usages = state.sessionTokenUsage.entries.sortedByDescending { it.value.estimatedCostUsd }
        if (usages.isEmpty()) {
            item { SectionEmpty("暂无会话用量记录") }
        } else {
            val titleById = state.sessions.associate { it.id to (it.title.ifBlank { "Untitled" }) }
            items(usages.toList(), key = { it.key }) { (sessionId, usage) ->
                Row(
                    modifier = Modifier
                        .fillMaxWidth()
                        .clip(RoundedCornerShape(10.dp))
                        .padding(vertical = 2.dp),
                    horizontalArrangement = Arrangement.SpaceBetween,
                    verticalAlignment = androidx.compose.ui.Alignment.CenterVertically,
                ) {
                    Column(Modifier.weight(1f)) {
                        Text(
                            text = titleById[sessionId] ?: sessionId.take(12),
                            style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                            fontWeight = FontWeight.Medium,
                            color = scheme.onSurface,
                            maxLines = 1,
                        )
                        Text(
                            text = usage.formattedTokens,
                            style = MonoStyle,
                            color = scheme.onSurfaceVariant,
                        )
                    }
                    Text(
                        text = "${usage.totalTokens} tokens",
                        style = MonoStyle,
                        color = scheme.onSurfaceVariant,
                    )
                }
            }
        }
    }
}

@Composable
private fun StatItem(label: String, value: String, color: androidx.compose.ui.graphics.Color, modifier: Modifier = Modifier) {
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(4.dp)) {
        Text(
            text = value,
            style = MaterialTheme.typography.titleLarge,
            fontWeight = FontWeight.SemiBold,
            color = color,
        )
        Text(
            text = label,
            style = MaterialTheme.typography.labelMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f),
        )
    }
}

private fun formatCount(n: Long): String = when {
    n >= 1_000_000 -> "%.1fM".format(n / 1_000_000.0)
    n >= 1_000 -> "%.1fk".format(n / 1_000.0)
    else -> n.toString()
}
