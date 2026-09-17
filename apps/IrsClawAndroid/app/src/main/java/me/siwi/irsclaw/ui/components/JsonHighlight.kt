package me.siwi.irsclaw.ui.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.FontWeight
import me.siwi.irsclaw.ui.theme.IosColors

/**
 * Hand-rolled JSON tokenizer producing a syntax-highlighted [AnnotatedString] —
 * the Compose port of the iOS JSONHighlightView: keys purple, strings green,
 * numbers blue, booleans orange, null red, punctuation secondary.
 */
@Composable
fun jsonColors(): JsonColors {
    val punctuation = MaterialTheme.colorScheme.onSurfaceVariant
    return remember(punctuation) {
        JsonColors(
            key = IosColors.Purple,
            string = IosColors.Green,
            number = IosColors.Blue,
            boolean = IosColors.Orange,
            nullColor = IosColors.Red,
            plain = punctuation,
        )
    }
}

data class JsonColors(
    val key: Color,
    val string: Color,
    val number: Color,
    val boolean: Color,
    val nullColor: Color,
    val plain: Color,
)

fun highlightJson(text: String, colors: JsonColors): AnnotatedString = buildAnnotatedString {
    var i = 0
    val n = text.length
    while (i < n) {
        val c = text[i]
        when {
            c == '"' -> {
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
                appendAnnotated(
                    text.substring(i, end + 1),
                    SpanStyle(color = if (isKey) colors.key else colors.string, fontWeight = if (isKey) FontWeight.Medium else null),
                )
                i = end + 1
            }
            c.isDigit() || (c == '-' && i + 1 < n && text[i + 1].isDigit()) -> {
                var j = i + 1
                while (j < n && (text[j].isDigit() || text[j] == '.' || text[j] == 'e' || text[j] == 'E' || text[j] == '+' || text[j] == '-')) j++
                appendAnnotated(text.substring(i, j), SpanStyle(color = colors.number))
                i = j
            }
            text.startsWith("true", i) || text.startsWith("false", i) -> {
                appendAnnotated(text.substring(i, i + if (text.startsWith("true", i)) 4 else 5), SpanStyle(color = colors.boolean, fontWeight = FontWeight.Medium))
                i += if (text.startsWith("true", i)) 4 else 5
            }
            text.startsWith("null", i) -> {
                appendAnnotated("null", SpanStyle(color = colors.nullColor))
                i += 4
            }
            c == '{' || c == '}' || c == '[' || c == ']' || c == ',' || c == ':' -> {
                appendAnnotated(c.toString(), SpanStyle(color = colors.plain))
                i++
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
