package me.siwi.irsclaw.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Help
import androidx.compose.material.icons.filled.Bolt
import androidx.compose.material.icons.filled.Extension
import androidx.compose.material.icons.filled.Image
import androidx.compose.material.icons.filled.Person
import androidx.compose.material.icons.filled.Psychology
import androidx.compose.material.icons.filled.RateReview
import androidx.compose.material.icons.filled.Star
import androidx.compose.material.icons.filled.ThumbUp
import androidx.compose.material.icons.outlined.Error
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.data.model.AppMessage

private data class AvatarStyle(val from: Color, val to: Color, val icon: ImageVector)

private fun styleFor(message: AppMessage): AvatarStyle = when (message) {
    is AppMessage.User -> AvatarStyle(Color(0xFF007AFF), Color(0xFF00C7BE), Icons.Filled.Person)
    is AppMessage.Assistant -> AvatarStyle(Color(0xFFAF52DE), Color(0xFFFF2D92), Icons.Filled.Bolt)
    is AppMessage.ToolCall -> AvatarStyle(Color(0xFFFF9500), Color(0xFFFFD60A), Icons.Filled.Extension)
    is AppMessage.Reasoning -> AvatarStyle(Color(0xFF5E5CE6), Color(0xFF64D2FF), Icons.Filled.Psychology)
    is AppMessage.Error -> AvatarStyle(Color(0xFFFF3B30), Color(0xFFFF9500), Icons.Outlined.Error)
    is AppMessage.Status -> AvatarStyle(Color(0xFF8E8E93), Color(0xFFAEAEB2), Icons.AutoMirrored.Filled.Help)
    is AppMessage.Evaluation -> AvatarStyle(Color(0xFF30D158), Color(0xFF64D2FF), Icons.Filled.RateReview)
    is AppMessage.Quality -> AvatarStyle(Color(0xFFFFD60A), Color(0xFFFF9500), Icons.Filled.Star)
    is AppMessage.Feedback -> AvatarStyle(Color(0xFF30D158), Color(0xFF64D2FF), Icons.Filled.ThumbUp)
    is AppMessage.Image -> AvatarStyle(Color(0xFF64D2FF), Color(0xFF0A84FF), Icons.Filled.Image)
}

/** Left gradient avatar identifying each bubble type, mirroring the iOS AvatarView. */
@Composable
fun AvatarView(message: AppMessage, size: Dp = 28.dp) {
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
            modifier = Modifier.size(size * 0.55f),
        )
    }
}
