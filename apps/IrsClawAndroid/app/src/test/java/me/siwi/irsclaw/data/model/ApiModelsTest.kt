package me.siwi.irsclaw.data.model

import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class ApiModelsTest {

    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun `decodes session list with snake_case keys`() {
        val body = """
            {"success":true,"data":[
              {"id":"s1","title":"Hello world","message_count":3,"created_at":1758000000,"agent_id":"default"}
            ],"error":null}
        """.trimIndent()
        val sessions = json.decodeFromString<ApiResponse<List<SessionMeta>>>(body)
        assertTrue(sessions.success)
        val s = sessions.data!!.single()
        assertEquals("s1", s.id)
        assertEquals(3, s.messageCount)
        assertEquals(1758000000L, s.createdAt)
        assertEquals("default", s.agentId)
    }

    @Test
    fun `decodes openai style tool schema`() {
        val body = """
            {"success":true,"data":[
              {"type":"function","function":{"name":"web_search","description":"Search the web"}},
              {"name":"flat_tool","description":"flat fallback"}
            ]}
        """.trimIndent()
        val tools = json.decodeFromString<ApiResponse<List<ToolInfo>>>(body).data!!
        assertEquals(2, tools.size)
        assertEquals("web_search", tools[0].name)
        assertEquals("Search the web", tools[0].description)
        assertEquals("flat_tool", tools[1].name)
    }

    @Test
    fun `decodes done payload with usage and quality`() {
        val data = """
            {"usage":{"prompt_tokens":120,"completion_tokens":45,"total_tokens":165,"estimated_cost_usd":0.0021},
             "quality":{"score":"good","complete":true,"issues":[],"references_valid":true},
             "session_id":"sess-9"}
        """.trimIndent()
        val done = json.decodeFromString<DonePayload>(data)
        assertEquals(120, done.usage!!.promptTokens)
        assertEquals(165, done.usage!!.serverTotalTokens)
        assertEquals("sess-9", done.sessionId)
        assertEquals("good", done.quality!!.score)
        assertTrue(done.quality!!.complete)
        assertTrue(done.quality!!.referencesValid == true)
    }

    @Test
    fun `token usage formatting matches iOS`() {
        val usage = TokenUsage(promptTokens = 1200, completionTokens = 345, serverTotalTokens = 1545, estimatedCostUsd = 0.0042)
        assertEquals("↑1.2k ↓345", usage.formattedTokens)
        assertEquals(1545, usage.totalTokens)
        assertEquals("$0.0042", usage.formattedCost)

        val sum = usage + TokenUsage(100, 55, 155, 0.0008)
        assertEquals(1300, sum.promptTokens)
        assertEquals(0.005, sum.estimatedCostUsd, 1e-9)
    }

    @Test
    fun `claw message decodes every role variant`() {
        val body = """
            {"success":true,"data":[
              {"role":"user","content":"hi"},
              {"role":"tool_call","name":"i_rs","args":"{}","result":"ok"},
              {"role":"evaluation","tool":"i_rs","valid":false,"issues":["bad arg"]},
              {"role":"quality","score":"fair","complete":false,"issues":["x"],"references_valid":false},
              {"role":"feedback","positive":true,"message":"nice"},
              {"role":"image","alt_text":"a chart","width":512,"height":512,"format":"png","url":"/api/images/a.png"}
            ]}
        """.trimIndent()
        val messages = json.decodeFromString<ApiResponse<List<ClawMessage>>>(body).data!!
        assertEquals(6, messages.size)
        assertNull(messages[1].reasoning)
        val items = messages.toMessageItems()
        // tool_call has no reasoning; image maps to Image bubble
        assertEquals(6, items.size)
        assertTrue(items[0].message is AppMessage.User)
        assertTrue(items[1].message is AppMessage.ToolCall)
        assertTrue(items[2].message is AppMessage.Evaluation)
        assertTrue(items[3].message is AppMessage.Quality)
        assertTrue(items[4].message is AppMessage.Feedback)
        assertTrue(items[5].message is AppMessage.Image)
    }

    @Test
    fun `history reasoning becomes separate bubble`() {
        val msg = json.decodeFromString<ClawMessage>(
            """{"role":"assistant","content":"answer","reasoning":"let me think"}""",
        )
        val items = msg.toMessageItems()
        assertEquals(2, items.size)
        assertTrue(items[0].message is AppMessage.Reasoning)
        assertEquals("let me think", (items[0].message as AppMessage.Reasoning).text)
        assertTrue(items[1].message is AppMessage.Assistant)
    }
}
