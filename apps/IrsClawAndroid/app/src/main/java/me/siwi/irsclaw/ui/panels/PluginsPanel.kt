package me.siwi.irsclaw.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Extension
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.siwi.irsclaw.data.model.PluginInfo
import me.siwi.irsclaw.ui.components.CardChrome
import me.siwi.irsclaw.ui.theme.IosColors

/** Plugin cards (iOS PluginsPanel): blue→cyan tile, version chip, Active/Off pill. */
@Composable
fun PluginsPanel(plugins: List<PluginInfo>, modifier: Modifier = Modifier) {
    if (plugins.isEmpty()) {
        SectionEmpty("没有已发现插件", modifier)
        return
    }
    val dark = isSystemInDarkTheme()
    LazyColumn(
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(20.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        items(plugins, key = { it.name }) { plugin ->
            PluginCard(plugin, dark)
        }
    }
}

@Composable
private fun PluginCard(plugin: PluginInfo, dark: Boolean) {
    val scheme = MaterialTheme.colorScheme
    val borderBrush = if (plugin.enabled) {
        CardChrome.border(IosColors.Blue, dark)
    } else {
        Brush.linearGradient(
            listOf(
                scheme.onSurfaceVariant.copy(alpha = if (dark) 0.15f else 0.1f),
                scheme.onSurfaceVariant.copy(alpha = if (dark) 0.1f else 0.05f),
            ),
        )
    }
    Row(
        modifier = Modifier
            .fillMaxWidth()
            .shadow(8.dp, RoundedCornerShape(14.dp), spotColor = CardChrome.shadowColor())
            .background(CardChrome.background(), RoundedCornerShape(14.dp))
            .border(1.dp, borderBrush, RoundedCornerShape(14.dp))
            .padding(14.dp),
        horizontalArrangement = Arrangement.spacedBy(14.dp),
    ) {
        Box(
            modifier = Modifier
                .size(44.dp)
                .shadow(
                    if (plugin.enabled) 4.dp else 0.dp,
                    RoundedCornerShape(10.dp),
                    spotColor = IosColors.Blue.copy(alpha = if (dark) 0.4f else 0.3f),
                )
                .background(
                    if (plugin.enabled) {
                        Brush.linearGradient(listOf(IosColors.Blue, IosColors.Cyan))
                    } else {
                        Brush.linearGradient(
                            listOf(
                                scheme.onSurfaceVariant.copy(alpha = if (dark) 0.5f else 0.4f),
                                scheme.onSurfaceVariant.copy(alpha = if (dark) 0.35f else 0.25f),
                            ),
                        )
                    },
                    RoundedCornerShape(10.dp),
                ),
            contentAlignment = Alignment.Center,
        ) {
            Icon(Icons.Filled.Extension, contentDescription = null, tint = Color.White, modifier = Modifier.size(16.dp))
        }
        Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(3.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                Text(
                    text = plugin.name,
                    style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
                    fontWeight = FontWeight.Medium,
                    color = scheme.onSurface,
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
                if (plugin.version.isNotBlank()) {
                    Text(
                        text = "v${plugin.version}",
                        style = MaterialTheme.typography.labelSmall,
                        color = scheme.onSurfaceVariant,
                        modifier = Modifier
                            .background(scheme.onSurfaceVariant.copy(alpha = if (dark) 0.15f else 0.1f), RoundedCornerShape(4.dp))
                            .padding(horizontal = 6.dp, vertical = 2.dp),
                    )
                }
            }
            Text(
                text = plugin.description,
                style = MaterialTheme.typography.bodySmall,
                color = scheme.onSurfaceVariant,
                maxLines = 2,
                overflow = TextOverflow.Ellipsis,
            )
            plugin.author?.let {
                Text(it, style = MaterialTheme.typography.labelSmall, color = scheme.onSurfaceVariant.copy(alpha = 0.7f))
            }
        }
        Text(
            text = if (plugin.enabled) "Active" else "Off",
            style = MaterialTheme.typography.bodySmall,
            fontWeight = FontWeight.Medium,
            color = if (plugin.enabled) IosColors.Green else scheme.onSurfaceVariant,
            modifier = Modifier
                .align(Alignment.CenterVertically)
                .background(
                    if (plugin.enabled) {
                        Brush.linearGradient(
                            listOf(
                                IosColors.Green.copy(alpha = if (dark) 0.25f else 0.15f),
                                IosColors.Green.copy(alpha = if (dark) 0.15f else 0.08f),
                            ),
                        )
                    } else {
                        Brush.linearGradient(
                            listOf(
                                scheme.onSurfaceVariant.copy(alpha = if (dark) 0.15f else 0.1f),
                                scheme.onSurfaceVariant.copy(alpha = if (dark) 0.1f else 0.05f),
                            ),
                        )
                    },
                    RoundedCornerShape(50),
                )
                .padding(horizontal = 10.dp, vertical = 5.dp),
        )
    }
}
