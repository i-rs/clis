package me.siwi.irsclaw.ui.components

import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.scale
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.ui.theme.IosColors

/**
 * Red recording dot with a pulsing halo — the iOS PulsingDot: inner 6pt red
 * circle, outer red@0.2 halo scaling 1→1.4 (easeInOut 0.8s, autoreversing).
 */
@Composable
fun PulsingDot(size: Dp = 6.dp, color: Color = IosColors.Red, modifier: Modifier = Modifier) {
    val transition = rememberInfiniteTransition(label = "pulsing")
    val haloScale by transition.animateFloat(
        initialValue = 1f,
        targetValue = 1.4f,
        animationSpec = infiniteRepeatable(tween(800), RepeatMode.Reverse),
        label = "halo",
    )
    Box(modifier = modifier.size(size * 2.4f), contentAlignment = Alignment.Center) {
        Canvas(Modifier.size(size * 2.4f).scale(haloScale)) {
            drawCircle(color.copy(alpha = 0.2f), radius = this.size.minDimension / 2f, center = Offset(x = this.size.width / 2f, y = this.size.height / 2f))
        }
        Canvas(Modifier.size(size)) {
            drawCircle(color)
        }
    }
}
