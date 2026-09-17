package me.siwi.irsclaw.ui.components

import android.text.method.LinkMovementMethod
import android.widget.TextView
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import io.noties.markwon.Markwon
import io.noties.markwon.ext.tables.TablePlugin
import io.noties.markwon.ext.tables.TableTheme

/**
 * Markdown renderer for assistant bubbles — the port of the iOS MarkdownTextView
 * (MarkdownUI Theme.gitHub base): GitHub-style code background and the iOS table
 * palette. Markwon tolerates incomplete markdown, which is what streams in.
 */
@Composable
fun MarkdownText(markdown: String, modifier: Modifier = Modifier) {
    val context = LocalContext.current
    val dark = androidx.compose.foundation.isSystemInDarkTheme()
    val markwon = remember(context, dark) {
        val border = if (dark) 0xFF42444E.toInt() else 0xFFE4E4E8.toInt()
        val header = if (dark) 0xFF25262A.toInt() else 0xFFF7F7F9.toInt()
        val odd = if (dark) 0xFF18191D.toInt() else 0xFFFFFFFF.toInt()
        val even = if (dark) 0xFF25262A.toInt() else 0xFFF7F7F9.toInt()
        val codeBg = if (dark) 0xFF1C1F26.toInt() else 0xFFF6F8FA.toInt()
        val tableTheme = TableTheme.Builder()
            .tableBorderColor(border)
            .tableHeaderRowBackgroundColor(header)
            .tableOddRowBackgroundColor(odd)
            .tableEvenRowBackgroundColor(even)
            .build()
        Markwon.builder(context)
            .usePlugin(TablePlugin.create(tableTheme))
            .build()
    }
    val textColor = MaterialTheme.colorScheme.onSurface
    AndroidView(
        modifier = modifier,
        factory = { ctx ->
            TextView(ctx).apply {
                movementMethod = LinkMovementMethod.getInstance()
                textSize = 15f
                setLineSpacing(0f, 1.25f)
            }
        },
        update = { tv ->
            tv.setTextColor(textColor.toArgb())
            markwon.setMarkdown(tv, markdown)
        },
    )
}
