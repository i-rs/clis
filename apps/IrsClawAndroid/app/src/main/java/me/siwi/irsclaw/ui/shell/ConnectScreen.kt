package me.siwi.irsclaw.ui.shell

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Link
import androidx.compose.material3.Button
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import me.siwi.irsclaw.logic.ConnectionState
import me.siwi.irsclaw.ui.components.EmptyState

/**
 * Connection gate shown while disconnected/failed/waiting — the port of the iOS
 * "Connect to i-rs-claw Backend" screen, with a settings entry so the backend
 * URL/token can be configured before connecting.
 */
@Composable
fun ConnectScreen(
    state: ConnectionState,
    errorMessage: String?,
    backendName: String,
    onConnect: () -> Unit,
    onOpenSettings: (() -> Unit)? = null,
    modifier: Modifier = Modifier,
) {
    Surface(modifier = modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
        when (state) {
            ConnectionState.WAITING_FOR_HEALTH -> Column(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                CircularProgressIndicator(modifier = Modifier.size(32.dp))
                Text(
                    text = "Connecting to $backendName…",
                    style = MaterialTheme.typography.bodyMedium,
                    color = MaterialTheme.colorScheme.onSurfaceVariant,
                    modifier = Modifier.padding(top = 12.dp),
                )
            }

            ConnectionState.CONNECTED -> Unit

            else -> Column(
                modifier = Modifier.fillMaxSize(),
                verticalArrangement = Arrangement.Center,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                EmptyState(
                    icon = Icons.Filled.Link,
                    title = "Connect to i-rs-claw Backend",
                    message = errorMessage
                        ?: "Make sure claw serve is running and reachable at $backendName",
                )
                Button(
                    onClick = onConnect,
                    modifier = Modifier.padding(top = 8.dp),
                ) {
                    Text(if (state == ConnectionState.FAILED) "Retry" else "Connect")
                }
                if (onOpenSettings != null) {
                    TextButton(onClick = onOpenSettings) {
                        Text("Server Settings")
                    }
                }
            }
        }
    }
}
