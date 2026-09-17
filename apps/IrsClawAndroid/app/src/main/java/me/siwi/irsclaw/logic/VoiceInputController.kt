package me.siwi.irsclaw.logic

import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.speech.RecognitionListener
import android.speech.RecognizerIntent
import android.speech.SpeechRecognizer
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext

/**
 * Port of the iOS VoiceInputService: prefers zh-CN recognition with live partial
 * results. [onFinal] receives the finished transcript.
 */
class VoiceInputController(
    private val context: Context,
    private val onFinal: (String) -> Unit,
) {
    var isListening by mutableStateOf(false)
        private set
    var partialText by mutableStateOf("")
        private set
    var error by mutableStateOf<String?>(null)
        private set

    private var recognizer: SpeechRecognizer? = null

    private val listener = object : RecognitionListener {
        override fun onReadyForSpeech(params: Bundle?) {
            error = null
        }

        override fun onBeginningOfSpeech() {}

        override fun onRmsChanged(rmsdB: Float) {}

        override fun onBufferReceived(buffer: ByteArray?) {}

        override fun onEndOfSpeech() {}

        override fun onError(errorCode: Int) {
            isListening = false
            error = when (errorCode) {
                SpeechRecognizer.ERROR_NO_MATCH -> "没有听到说话"
                SpeechRecognizer.ERROR_SPEECH_TIMEOUT -> "没有听到说话"
                SpeechRecognizer.ERROR_INSUFFICIENT_PERMISSIONS -> "需要麦克风权限"
                SpeechRecognizer.ERROR_RECOGNIZER_BUSY -> "识别器忙碌，请稍后再试"
                else -> "语音识别出错 ($errorCode)"
            }
        }

        override fun onResults(results: Bundle?) {
            isListening = false
            partialText = ""
            val text = results
                ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                ?.firstOrNull()
            if (!text.isNullOrBlank()) onFinal(text)
        }

        override fun onPartialResults(partialResults: Bundle?) {
            partialText = partialResults
                ?.getStringArrayList(SpeechRecognizer.RESULTS_RECOGNITION)
                ?.firstOrNull()
                ?: ""
        }

        override fun onEvent(eventType: Int, params: Bundle?) {}
    }

    fun start() {
        if (isListening) return
        if (!SpeechRecognizer.isRecognitionAvailable(context)) {
            error = "此设备不支持语音识别"
            return
        }
        error = null
        partialText = ""
        recognizer?.destroy()
        recognizer = SpeechRecognizer.createSpeechRecognizer(context).apply {
            setRecognitionListener(listener)
            startListening(
                Intent(RecognizerIntent.ACTION_RECOGNIZE_SPEECH).apply {
                    putExtra(RecognizerIntent.EXTRA_LANGUAGE_MODEL, RecognizerIntent.LANGUAGE_MODEL_FREE_FORM)
                    putExtra(RecognizerIntent.EXTRA_LANGUAGE, "zh-CN")
                    putExtra(RecognizerIntent.EXTRA_PARTIAL_RESULTS, true)
                },
            )
        }
        isListening = true
    }

    /** Finish listening and flush whatever was recognized (iOS "Done"). */
    fun finish() {
        recognizer?.stopListening()
    }

    fun cancel() {
        recognizer?.destroy()
        recognizer = null
        isListening = false
        partialText = ""
    }
}

/** Remembers a [VoiceInputController] bound to the composition lifecycle. */
@Composable
fun rememberVoiceInput(onFinal: (String) -> Unit): VoiceInputController {
    val context = LocalContext.current
    val controller = remember(context) { VoiceInputController(context, onFinal) }
    DisposableEffect(Unit) {
        onDispose { controller.cancel() }
    }
    return controller
}
