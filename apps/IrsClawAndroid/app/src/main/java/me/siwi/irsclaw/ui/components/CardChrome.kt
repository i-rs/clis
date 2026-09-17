package me.siwi.irsclaw.ui.components

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.graphics.Color

/**
 * iOS panel card chrome: light #FCFCFC→#F7F7F7, dark #2E2E2E→#262626 gradient
 * background with a soft shadow and a 1pt accent-tinted gradient border.
 */
object CardChrome {
    @Composable
    fun background(): Brush {
        val dark = isSystemInDarkTheme()
        return if (dark) {
            Brush.linearGradient(listOf(Color(0xFF2E2E2E), Color(0xFF262626)))
        } else {
            Brush.linearGradient(listOf(Color(0xFFFCFCFC), Color(0xFFF7F7F7)))
        }
    }

    @Composable
    fun shadowColor(): Color = if (isSystemInDarkTheme()) Color.Black.copy(alpha = 0.3f) else Color.Black.copy(alpha = 0.05f)

    fun border(accent: Color, dark: Boolean): Brush = if (dark) {
        Brush.linearGradient(listOf(accent.copy(alpha = 0.25f), accent.copy(alpha = 0.1f)))
    } else {
        Brush.linearGradient(listOf(accent.copy(alpha = 0.15f), accent.copy(alpha = 0.05f)))
    }

    fun tile(accent: Color, dark: Boolean): Brush = if (dark) {
        Brush.linearGradient(listOf(accent.copy(alpha = 0.3f), accent.copy(alpha = 0.2f)))
    } else {
        Brush.linearGradient(listOf(accent.copy(alpha = 0.2f), accent.copy(alpha = 0.1f)))
    }
}
