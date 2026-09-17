package me.siwi.irsclaw

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.runtime.getValue
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import androidx.compose.material3.windowsizeclass.ExperimentalMaterial3WindowSizeClassApi
import androidx.compose.material3.windowsizeclass.calculateWindowSizeClass
import me.siwi.irsclaw.data.settings.Appearance
import me.siwi.irsclaw.logic.ClawViewModel
import me.siwi.irsclaw.ui.shell.AppShell
import me.siwi.irsclaw.ui.theme.IrsClawTheme

class MainActivity : ComponentActivity() {

    @OptIn(ExperimentalMaterial3WindowSizeClassApi::class)
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            val viewModel: ClawViewModel = viewModel()
            val settings by viewModel.settings.collectAsStateWithLifecycle()
            val widthSizeClass = calculateWindowSizeClass(this).widthSizeClass

            IrsClawTheme(appearance = settings?.appearance ?: Appearance.AUTO) {
                AppShell(viewModel = viewModel, windowWidth = widthSizeClass)
            }
        }
    }
}
