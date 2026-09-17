package me.siwi.irsclaw.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Star
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.foundation.shape.CircleShape

/**
 * iOS AgentChip: radius-8 capsule, active = accent fill with white semibold
 * label and a translucent inner circle + star; inactive = gray fill with border.
 */
@Composable
fun AgentChip(
    agentId: String,
    selected: Boolean,
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
) {
    val scheme = MaterialTheme.colorScheme
    Row(
        modifier = modifier
            .background(
                if (selected) scheme.primary else scheme.onSurfaceVariant.copy(alpha = 0.1f),
                RoundedCornerShape(8.dp),
            )
            .border(
                0.5.dp,
                if (selected) Color.Transparent else scheme.onSurfaceVariant.copy(alpha = 0.2f),
                RoundedCornerShape(8.dp),
            )
            .clickable(onClick = onClick)
            .padding(horizontal = 10.dp, vertical = 6.dp),
        verticalAlignment = Alignment.CenterVertically,
        horizontalArrangement = Arrangement.spacedBy(6.dp),
    ) {
        androidx.compose.foundation.layout.Box(
            modifier = Modifier
                .size(20.dp)
                .background(Color.White.copy(alpha = if (selected) 0.3f else 0.15f), CircleShape),
            contentAlignment = Alignment.Center,
        ) {
            Icon(
                if (selected) Icons.Filled.Star else Icons.Filled.Person,
                contentDescription = null,
                tint = Color.White,
                modifier = Modifier.size(9.dp),
            )
        }
        Text(
            text = agentId,
            style = MaterialTheme.typography.bodySmall,
            fontWeight = if (selected) FontWeight.SemiBold else FontWeight.Normal,
            color = if (selected) Color.White else scheme.onSurface,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
