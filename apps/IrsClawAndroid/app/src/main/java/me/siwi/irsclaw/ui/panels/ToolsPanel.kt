package me.siwi.irsclaw.ui.panels

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.lazy.grid.GridCells
import androidx.compose.foundation.lazy.grid.LazyVerticalGrid
import androidx.compose.foundation.lazy.grid.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.Folder
import androidx.compose.material.icons.filled.InsertChart
import androidx.compose.material.icons.filled.Psychology
import androidx.compose.material.icons.filled.Search
import androidx.compose.material.icons.filled.Visibility
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.shadow
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import me.siwi.irsclaw.data.model.ToolInfo
import me.siwi.irsclaw.ui.components.CardChrome
import me.siwi.irsclaw.ui.theme.IosColors
import me.siwi.irsclaw.ui.theme.MonoStyle

private data class Category(val label: String, val color: Color, val icon: ImageVector)

private fun categoryOf(tool: ToolInfo): Category {
    val name = tool.name.lowercase()
    return when {
        name.startsWith("i_rs") || name.startsWith("i-rs") || name.startsWith("irs") ->
            Category("i-rs CLI", IosColors.Orange, Icons.Filled.Build)
        "search" in name || "web" in name || "fetch" in name ->
            Category("Search", IosColors.Blue, Icons.Filled.Search)
        "file" in name || "read" in name || "write" in name || "edit" in name ->
            Category("Files", IosColors.Teal, Icons.Filled.Folder)
        "memory" in name || "knowledge" in name ->
            Category("Memory", IosColors.Purple, Icons.Filled.Psychology)
        "vision" in name || "image" in name ->
            Category("Vision", IosColors.Indigo, Icons.Filled.Visibility)
        "chart" in name || "stats" in name ->
            Category("Visualization", IosColors.Green, Icons.Filled.InsertChart)
        else ->
            Category("Agent", IosColors.Pink, Icons.Filled.Build)
    }
}

/** Tools grid grouped by category with gradient cards (iOS ToolsPanel). */
@Composable
fun ToolsPanel(tools: List<ToolInfo>, modifier: Modifier = Modifier) {
    if (tools.isEmpty()) {
        SectionEmpty("没有可用工具", modifier)
        return
    }
    val dark = isSystemInDarkTheme()
    val grouped = remember(tools) { tools.groupBy { categoryOf(it) } }

    LazyVerticalGrid(
        columns = GridCells.Fixed(2),
        modifier = modifier.fillMaxSize(),
        contentPadding = androidx.compose.foundation.layout.PaddingValues(20.dp),
        horizontalArrangement = Arrangement.spacedBy(10.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        grouped.forEach { (category, categoryTools) ->
            item(key = "header_${category.label}") {
                Row(
                    modifier = Modifier
                        .fillMaxSize()
                        .padding(top = 8.dp),
                    horizontalArrangement = Arrangement.spacedBy(6.dp),
                    verticalAlignment = Alignment.CenterVertically,
                ) {
                    Icon(category.icon, contentDescription = null, tint = category.color, modifier = Modifier.size(11.dp))
                    Text(
                        text = category.label.uppercase(),
                        style = MaterialTheme.typography.labelMedium,
                        fontWeight = FontWeight.SemiBold,
                        color = MaterialTheme.colorScheme.onSurfaceVariant,
                    )
                    Text(
                        text = "${categoryTools.size}",
                        style = MaterialTheme.typography.labelSmall,
                        color = MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.7f),
                    )
                }
            }
            items(categoryTools, key = { it.name }) { tool ->
                ToolCard(tool, category, dark)
            }
        }
    }
}

@Composable
private fun ToolCard(tool: ToolInfo, category: Category, dark: Boolean) {
    val scheme = MaterialTheme.colorScheme
    Column(
        modifier = Modifier
            .shadow(8.dp, RoundedCornerShape(14.dp), spotColor = CardChrome.shadowColor())
            .background(CardChrome.background(), RoundedCornerShape(14.dp))
            .border(1.dp, CardChrome.border(category.color, dark), RoundedCornerShape(14.dp))
            .padding(14.dp),
        verticalArrangement = Arrangement.spacedBy(10.dp),
    ) {
        Box(
            modifier = Modifier
                .size(36.dp)
                .shadow(4.dp, RoundedCornerShape(10.dp), spotColor = category.color.copy(alpha = if (dark) 0.3f else 0.2f))
                .background(CardChrome.tile(category.color, dark), RoundedCornerShape(10.dp)),
            contentAlignment = Alignment.Center,
        ) {
            Icon(category.icon, contentDescription = null, tint = category.color, modifier = Modifier.size(14.dp))
        }
        Text(
            text = tool.name,
            style = MaterialTheme.typography.titleMedium.copy(fontSize = 15.sp),
            fontWeight = FontWeight.Medium,
            color = scheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
        Text(
            text = tool.description,
            style = MaterialTheme.typography.bodySmall,
            color = scheme.onSurface.copy(alpha = 0.7f),
            maxLines = 3,
            overflow = TextOverflow.Ellipsis,
        )
    }
}

@Composable
internal fun SectionEmpty(text: String, modifier: Modifier = Modifier) {
    Box(modifier = modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
        Text(
            text = text,
            style = MaterialTheme.typography.bodyMedium,
            color = MaterialTheme.colorScheme.onSurfaceVariant,
        )
    }
}
