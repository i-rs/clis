package me.siwi.irsclaw.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.sp
import me.siwi.irsclaw.data.settings.Appearance

/** iOS system palette literals used across the ported views. */
object IosColors {
    val Blue = Color(0xFF007AFF)
    val BlueDark = Color(0xFF0A84FF)
    val Cyan = Color(0xFF32ADE6)
    val Teal = Color(0xFF5AC8FA)
    val Mint = Color(0xFF00C7BE)
    val Purple = Color(0xFFAF52DE)
    val Pink = Color(0xFFFF2D55)
    val Orange = Color(0xFFFF9500)
    val Yellow = Color(0xFFFFCC00)
    val Green = Color(0xFF34C759)
    val GreenDark = Color(0xFF30D158)
    val Red = Color(0xFFFF3B30)
    val RedDark = Color(0xFFFF453A)
    val Indigo = Color(0xFF5856D6)
    val Gray = Color(0xFF8E8E93)
}

/** platformSecondaryBackground: #F2F2F7 light / #1C1C1E dark. */
val SecondaryBackgroundLight = Color(0xFFF2F2F7)
val SecondaryBackgroundDark = Color(0xFF1C1C1E)

/** platformTertiaryBackground: #FFFFFF light / #2C2C2E dark. */
val TertiaryBackgroundLight = Color(0xFFFFFFFF)
val TertiaryBackgroundDark = Color(0xFF2C2C2E)

private val LightColors = lightColorScheme(
    primary = IosColors.Blue,
    onPrimary = Color.White,
    primaryContainer = Color(0xFFD6E9FF),
    onPrimaryContainer = Color(0xFF001D36),
    secondary = IosColors.Gray,
    onSecondary = Color.White,
    secondaryContainer = SecondaryBackgroundLight,
    onSecondaryContainer = Color(0xFF1A1C1E),
    surface = Color(0xFFFFFFFF),
    onSurface = Color(0xFF1A1C1E),
    surfaceVariant = SecondaryBackgroundLight,
    onSurfaceVariant = IosColors.Gray,
    background = Color(0xFFFFFFFF),
    onBackground = Color(0xFF1A1C1E),
    surfaceContainer = SecondaryBackgroundLight,
    surfaceContainerHigh = TertiaryBackgroundLight,
    surfaceContainerHighest = Color(0xFFE5E5EA),
    outline = Color(0xFFC7C7CC),
    error = IosColors.Red,
)

private val DarkColors = darkColorScheme(
    primary = IosColors.BlueDark,
    onPrimary = Color.White,
    primaryContainer = Color(0xFF004882),
    onPrimaryContainer = Color(0xFFD1E4FF),
    secondary = IosColors.Gray,
    onSecondary = Color.White,
    secondaryContainer = SecondaryBackgroundDark,
    onSecondaryContainer = Color(0xFFE2E2E5),
    surface = Color.Black,
    onSurface = Color(0xFFE2E2E5),
    surfaceVariant = SecondaryBackgroundDark,
    onSurfaceVariant = IosColors.Gray,
    background = Color.Black,
    onBackground = Color(0xFFE2E2E5),
    surfaceContainer = SecondaryBackgroundDark,
    surfaceContainerHigh = TertiaryBackgroundDark,
    surfaceContainerHighest = Color(0xFF3A3A3C),
    outline = Color(0xFF48484A),
    error = IosColors.RedDark,
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

/** Monospaced style for JSON args/results, tokens and prompts (iOS .monospaced). */
val MonoStyle = TextStyle(
    fontFamily = FontFamily.Monospace,
    fontSize = 12.sp,
)
