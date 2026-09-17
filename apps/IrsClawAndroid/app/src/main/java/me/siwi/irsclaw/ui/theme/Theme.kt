package me.siwi.irsclaw.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.unit.sp
import me.siwi.irsclaw.data.settings.Appearance

// Brand palette — mirrors the iOS system-blue accent and its dark counterpart.
private val Accent = Color(0xFF0A84FF)
private val AccentDark = Color(0xFF409CFF)

private val LightColors = lightColorScheme(
    primary = Color(0xFF007AFF),
    onPrimary = Color.White,
    primaryContainer = Color(0xFFD6E9FF),
    onPrimaryContainer = Color(0xFF001D36),
    secondary = Color(0xFF516070),
    onSecondary = Color.White,
    secondaryContainer = Color(0xFFD5E4F7),
    onSecondaryContainer = Color(0xFF0E1D2A),
    surface = Color(0xFFFDFDFE),
    onSurface = Color(0xFF1A1C1E),
    surfaceVariant = Color(0xFFF0F1F4),
    onSurfaceVariant = Color(0xFF43474E),
    background = Color(0xFFFDFDFE),
    onBackground = Color(0xFF1A1C1E),
    surfaceContainer = Color(0xFFF5F6F8),
    surfaceContainerHigh = Color(0xFFEFF0F3),
    outline = Color(0xFF73777F),
    error = Color(0xFFBA1A1A),
)

private val DarkColors = darkColorScheme(
    primary = AccentDark,
    onPrimary = Color(0xFF00315C),
    primaryContainer = Color(0xFF004882),
    onPrimaryContainer = Color(0xFFD1E4FF),
    secondary = Color(0xFFB9C8DA),
    onSecondary = Color(0xFF243240),
    secondaryContainer = Color(0xFF3A4857),
    onSecondaryContainer = Color(0xFFD5E4F7),
    surface = Color(0xFF111417),
    onSurface = Color(0xFFE2E2E5),
    surfaceVariant = Color(0xFF1C1F23),
    onSurfaceVariant = Color(0xFFC3C6CD),
    background = Color(0xFF0E1013),
    onBackground = Color(0xFFE2E2E5),
    surfaceContainer = Color(0xFF171A1E),
    surfaceContainerHigh = Color(0xFF1E2126),
    outline = Color(0xFF8D9199),
    error = Color(0xFFFFB4AB),
)

@Composable
fun IrsClawTheme(
    appearance: Appearance = Appearance.AUTO,
    content: @Composable () -> Unit,
) {
    val dark = when (appearance) {
        Appearance.LIGHT -> false
        Appearance.DARK -> true
        Appearance.AUTO -> isSystemInDarkTheme()
    }
    MaterialTheme(
        colorScheme = if (dark) DarkColors else LightColors,
        typography = IrsClawTypography,
        content = content,
    )
}

val IrsClawTypography = Typography(
    headlineMedium = TextStyle(fontWeight = FontWeight.SemiBold, fontSize = 26.sp),
    titleLarge = TextStyle(fontWeight = FontWeight.SemiBold, fontSize = 20.sp),
    titleMedium = TextStyle(fontWeight = FontWeight.SemiBold, fontSize = 16.sp),
    bodyLarge = TextStyle(fontSize = 16.sp),
    bodyMedium = TextStyle(fontSize = 14.sp),
    bodySmall = TextStyle(fontSize = 12.sp),
    labelLarge = TextStyle(fontWeight = FontWeight.Medium, fontSize = 14.sp),
    labelMedium = TextStyle(fontWeight = FontWeight.Medium, fontSize = 12.sp),
    labelSmall = TextStyle(fontWeight = FontWeight.Medium, fontSize = 10.sp),
)

/** Monospaced style for JSON args/results, tokens and prompts (iOS uses .monospaced). */
val MonoStyle = TextStyle(
    fontFamily = FontFamily.Monospace,
    fontSize = 12.sp,
)
