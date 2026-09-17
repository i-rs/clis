package me.siwi.irsclaw.data.api

/**
 * Incremental SSE line parser — a faithful port of the iOS byte-level state machine.
 *
 * Feed each line as it arrives (without the trailing newline); when a non-null
 * [SseEvent] comes back, dispatch it. Rules mirrored from the iOS client:
 *  - `event:` sets the event name (whitespace-trimmed).
 *  - `data:` appends the payload; an optional single leading space is stripped and,
 *    per the SSE spec, multiple `data:` lines of one event are joined with `\n`
 *    (the axum backend preserves original newlines inside payloads).
 *  - An empty line dispatches the pending event and resets state.
 *  - `id:`, `retry:` and comment lines are ignored.
 */
class SseParser {
    data class SseEvent(val event: String, val data: String)

    private var currentEvent: String = ""
    private var currentData: StringBuilder = StringBuilder()

    fun feed(line: String): SseEvent? = when {
        line.startsWith("event:") -> {
            currentEvent = line.substring(6).trim()
            null
        }
        line.startsWith("data:") -> {
            val raw = line.substring(5)
            val chunk = raw.removePrefix(" ")
            if (currentEvent.isNotEmpty() || chunk.isNotEmpty()) {
                if (currentData.isEmpty()) currentData.append(chunk) else currentData.append('\n').append(chunk)
            }
            null
        }
        line.isEmpty() -> {
            val event = if (currentEvent.isNotEmpty()) SseEvent(currentEvent, currentData.toString()) else null
            currentEvent = ""
            currentData.clear()
            event
        }
        else -> null
    }

    fun reset() {
        currentEvent = ""
        currentData.clear()
    }
}
