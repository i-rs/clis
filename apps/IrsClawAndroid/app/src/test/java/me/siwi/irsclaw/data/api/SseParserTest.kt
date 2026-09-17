package me.siwi.irsclaw.data.api

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test

class SseParserTest {

    private fun parse(vararg lines: String): List<SseParser.SseEvent> {
        val parser = SseParser()
        return lines.mapNotNull { parser.feed(it) }
    }

    @Test
    fun `single token event with one data line`() {
        val events = parse(
            "event: token",
            "data: Hello",
            "",
        )
        assertEquals(listOf(SseParser.SseEvent("token", "Hello")), events)
    }

    @Test
    fun `multi-line data fields are joined with newline`() {
        val events = parse(
            "event: token",
            "data: first line",
            "data: second line",
            "",
        )
        assertEquals(1, events.size)
        assertEquals("first line\nsecond line", events[0].data)
    }

    @Test
    fun `leading space after data colon is stripped`() {
        val events = parse(
            "event: status",
            "data: thinking...",
            "",
        )
        assertEquals("thinking...", events.single().data)
    }

    @Test
    fun `id and comment lines are ignored`() {
        val events = parse(
            "id: 1",
            ": keepalive comment",
            "event: token",
            "data: ok",
            "",
        )
        assertEquals(listOf(SseParser.SseEvent("token", "ok")), events)
    }

    @Test
    fun `multiple events dispatched in order`() {
        val events = parse(
            "event: token",
            "data: 你好",
            "",
            "event: new_round",
            "data: ",
            "",
            "event: error",
            "data: boom",
            "",
        )
        assertEquals(3, events.size)
        assertEquals("token", events[0].event)
        assertEquals("你好", events[0].data)
        assertEquals("new_round", events[1].event)
        assertEquals("", events[1].data)
        assertEquals(SseParser.SseEvent("error", "boom"), events[2])
    }

    @Test
    fun `empty event name is not dispatched`() {
        val events = parse(
            "data: orphan",
            "",
        )
        assertEquals(0, events.size)
    }

    @Test
    fun `reset clears pending partial event`() {
        val parser = SseParser()
        assertNull(parser.feed("event: token"))
        assertNull(parser.feed("data: partial"))
        parser.reset()
        assertNull(parser.feed(""))
    }
}
