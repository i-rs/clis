package me.siwi.irsclaw.data.model

import kotlinx.serialization.KSerializer
import kotlinx.serialization.Serializable
import kotlinx.serialization.descriptors.SerialDescriptor
import kotlinx.serialization.encoding.Decoder
import kotlinx.serialization.encoding.Encoder
import kotlinx.serialization.json.JsonDecoder
import kotlinx.serialization.json.JsonEncoder
import kotlinx.serialization.json.JsonObject
import kotlinx.serialization.json.JsonPrimitive
import kotlinx.serialization.json.buildJsonObject
import kotlinx.serialization.json.decodeFromJsonElement
import kotlinx.serialization.json.jsonObject

/**
 * Tools arrive either as an OpenAI-style schema `{type, function: {name, description}}`
 * or as a flat `{name, description}`. Hand-rolled (not JsonTransformingSerializer)
 * because `@Serializable(with = ...)` plus a self-referencing surrogate serializer
 * hits a circular class-init at runtime.
 */
object ToolInfoSerializer : KSerializer<ToolInfo> {

    @Serializable
    private data class Flat(val name: String = "", val description: String = "")

    override val descriptor: SerialDescriptor = Flat.serializer().descriptor

    override fun deserialize(decoder: Decoder): ToolInfo {
        val jsonDecoder = decoder as JsonDecoder
        val obj = jsonDecoder.decodeJsonElement().jsonObject
        val source = (obj["function"] as? JsonObject) ?: obj
        val flat = jsonDecoder.json.decodeFromJsonElement(Flat.serializer(), source)
        return ToolInfo(flat.name, flat.description)
    }

    override fun serialize(encoder: Encoder, value: ToolInfo) {
        val jsonEncoder = encoder as JsonEncoder
        val element = buildJsonObject {
            put("type", JsonPrimitive("function"))
            put("name", JsonPrimitive(value.name))
            put("description", JsonPrimitive(value.description))
        }
        jsonEncoder.encodeJsonElement(element)
    }
}
