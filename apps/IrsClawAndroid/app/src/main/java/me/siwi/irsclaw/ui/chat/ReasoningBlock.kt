package me.siwi.irsclaw.ui.chat

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.spring
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Psychology
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.foundation.text.selection.SelectionContainer
import me.siwi.irsclaw.ui.theme.IosColors

/**
 * Collapsible "Thinking" block (iOS ReasoningBlock): the tool-card chrome with a
 * continuously pulsing brain icon, char count, and a 15pt expanded body.
 */
@Composable
fun ReasoningBlock(text: String) {
    var expanded by rememberSaveable { mutableStateOf(false) }
    val scheme = MaterialTheme.colorScheme
    val transition = rememberInfiniteTransition(label = "brain")
    val pulse by transition.animateFloat(
        initialValue = 0.45f,
        targetValue = 1f,
        animationSpec = infiniteRepeatable(tween(800), RepeatMode.Reverse),
        label = "pulse",
    )

    Column(
        modifier = Modifier
            .fillMaxWidth()
            .animateContentSize(spring(dampingRatio = 0.8f, stiffness = 380f))
            .background(scheme.surfaceVariant, RoundedCornerShape(14.dp))
            .border(1.dp, Color(0xFF8E8E93).copy(alpha = 0.15f), RoundedCornerShape(14.dp))
            .clickable { expanded = !expanded },
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Icon(
                Icons.Filled.Psychology,
                contentDescription = null,
                tint = IosColors.Indigo,
                modifier = Modifier
                    .size(12.dp)
                    .alpha(pulse),
            )
            Text(
                text = "Thinking",
                style = MaterialTheme.typography.bodySmall,
                fontWeight = FontWeight.Medium,
                color = scheme.onSurfaceVariant,
            )
            Spacer(Modifier.weight(1f))
            Text(
                text = if (text.length < 1000) "${text.length} chars" else "${text.length / 1000}k chars",
                style = MaterialTheme.typography.labelSmall,
                color = scheme.onSurfaceVariant.copy(alpha = 0.7f),
            )
            Icon(
                if (expanded) Icons.Filled.KeyboardArrowDown else Icons.Filled.KeyboardArrowRight,
                contentDescription = null,
                tint = scheme.onSurfaceVariant.copy(alpha = 0.7f),
                modifier = Modifier
                    .size(14.dp)
                    .width(12.dp),
            )
        }

        AnimatedVisibility(visible = expanded) {
            Column {
                androidx.compose.material3.HorizontalDivider(modifier = Modifier.padding(horizontal = 12.dp))
                SelectionContainer {
                    Text(
                        text = text,
                        style = MaterialTheme.typography.bodyLarge.copy(fontSize = 15.sp),
                        color = scheme.onSurface,
                        lineHeight = 21.sp,
                        modifier = Modifier.padding(horizontal = 14.dp, vertical = 12.dp),
                    )
                }
            }
        }
    }
}
