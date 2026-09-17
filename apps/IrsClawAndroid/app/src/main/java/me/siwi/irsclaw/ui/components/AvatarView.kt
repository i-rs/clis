package me.siwi.irsclaw.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AutoAwesome
import androidx.compose.material.icons.filled.Build
import androidx.compose.material.icons.filled.Cancel
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Photo
import androidx.compose.material.icons.filled.Psychology
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.ThumbUp
import androidx.compose.material.icons.filled.Verified
import androidx.compose.material3.Icon
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.data.model.AppMessage
import me.siwi.irsclaw.ui.theme.IosColors

private data class AvatarStyle(val from: Color, val to: Color, val icon: ImageVector)

// iOS AvatarView gradient pairs (36pt circle @ 0.9 scale → 32dp effective here).
private fun styleFor(message: AppMessage): AvatarStyle = when (message) {
    is AppMessage.User -> AvatarStyle(IosColors.Blue, IosColors.Cyan, Icons.Filled.Person)
    is AppMessage.Assistant -> AvatarStyle(IosColors.Purple, IosColors.Pink, Icons.Filled.AutoAwesome)
    is AppMessage.ToolCall -> AvatarStyle(IosColors.Orange, IosColors.Yellow, Icons.Filled.Build)
    is AppMessage.Reasoning -> AvatarStyle(IosColors.Indigo, IosColors.Teal, Icons.Filled.Psychology)
    is AppMessage.Error -> AvatarStyle(IosColors.Red, IosColors.Orange, Icons.Filled.Cancel)
    is AppMessage.Status -> AvatarStyle(IosColors.Gray, IosColors.Gray, Icons.Filled.AutoAwesome)
    is AppMessage.Evaluation ->
        if (message.valid) AvatarStyle(IosColors.Green, IosColors.Teal, Icons.Filled.Verified)
        else AvatarStyle(IosColors.Red, IosColors.Orange, Icons.Filled.Verified)
    is AppMessage.Quality -> AvatarStyle(IosColors.Yellow, IosColors.Orange, Icons.Filled.AutoAwesome)
    is AppMessage.Feedback ->
        if (message.positive) AvatarStyle(IosColors.Green, IosColors.Teal, Icons.Filled.ThumbUp)
        else AvatarStyle(IosColors.Red, IosColors.Orange, Icons.Filled.ThumbUp)
    is AppMessage.Image -> AvatarStyle(IosColors.Purple, IosColors.Pink, Icons.Filled.Photo)
}

/** Left gradient avatar identifying each bubble type (iOS AvatarView). */
@Composable
fun AvatarView(message: AppMessage, size: Dp = 32.dp) {
    val style = styleFor(message)
    Box(
        modifier = Modifier
            .size(size)
            .background(Brush.linearGradient(listOf(style.from, style.to)), CircleShape),
        contentAlignment = Alignment.Center,
    ) {
        Icon(
            imageVector = style.icon,
            contentDescription = null,
            tint = Color.White,
            modifier = Modifier.size(size * 0.42f),
        )
    }
}
