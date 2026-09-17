package me.siwi.irsclaw.ui.components

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.graphics.Color
import me.siwi.irsclaw.ui.theme.MonoStyle

/**
 * Hand-rolled JSON tokenizer producing a syntax-highlighted [AnnotatedString] —
 * the Compose port of the iOS JSONHighlightView (keys purple, strings green,
 * numbers blue, booleans/null orange).
 */
@Composable
fun jsonColors(): JsonColors {
    val dark = isSystemInDarkTheme()
    return remember(dark) {
        if (dark) {
            JsonColors(
                key = Color(0xFFBF5AF2),
                string = Color(0xFF30D158),
                number = Color(0xFF64D2FF),
                boolean = Color(0xFFFF9F0A),
                plain = Color(0xFFE2E2E5),
            )
        } else {
            JsonColors(
                key = Color(0xFFAF52DE),
                string = Color(0xFF248A3D),
                number = Color(0xFF0071E3),
                boolean = Color(0xFFC93400),
                plain = Color(0xFF1A1C1E),
            )
        }
    }
}

data class JsonColors(val key: Color, val string: Color, val number: Color, val boolean: Color, val plain: Color)

fun highlightJson(text: String, colors: JsonColors): AnnotatedString = buildAnnotatedString {
    var i = 0
    val n = text.length
    while (i < n) {
        val c = text[i]
        when {
            c == '"' -> {
                // String (key if followed by colon)
                var j = i + 1
                while (j < n && text[j] != '"') {
                    if (text[j] == '\\') j++
                    j++
                }
                val end = j.coerceAtMost(n - 1)
                val isKey = run {
                    var k = end + 1
                    while (k < n && text[k].isWhitespace()) k++
                    k < n && text[k] == ':'
                }
                appendAnnotated(text.substring(i, end + 1), SpanStyle(color = if (isKey) colors.key else colors.string, fontWeight = if (isKey) FontWeight.Medium else null))
                i = end + 1
            }
            c.isDigit() || (c == '-' && i + 1 < n && text[i + 1].isDigit()) -> {
                var j = i + 1
                while (j < n && (text[j].isDigit() || text[j] == '.' || text[j] == 'e' || text[j] == 'E' || text[j] == '+' || text[j] == '-')) j++
                appendAnnotated(text.substring(i, j), SpanStyle(color = colors.number))
                i = j
            }
            text.startsWith("true", i) || text.startsWith("false", i) || text.startsWith("null", i) -> {
                val word = if (text.startsWith("true", i) || text.startsWith("false", i)) {
                    text.substring(i, i + 4)
                } else {
                    "null"
                }
                appendAnnotated(word, SpanStyle(color = colors.boolean, fontWeight = FontWeight.Medium))
                i += word.length
            }
            else -> {
                appendAnnotated(c.toString(), SpanStyle(color = colors.plain))
                i++
            }
        }
    }
}

private fun AnnotatedString.Builder.appendAnnotated(text: String, style: SpanStyle) {
    val start = length
    append(text)
    addStyle(style, start, length)
}

/** Convenience: highlighted + monospaced JSON text. */
@Composable
fun rememberJsonHighlight(text: String): AnnotatedString {
    val colors = jsonColors()
    return remember(text, colors) { highlightJson(text, colors) }
}
