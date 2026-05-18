import Foundation
import Speech
import AVFoundation
import Combine

/// macOS voice input service using Apple Speech framework.
/// Press shortcut to record, release/button to transcribe into text.
@MainActor
class VoiceInputService: ObservableObject {
    @Published var isRecording = false
    @Published var transcribedText = ""
    @Published var errorMessage: String?

    private let audioEngine = AVAudioEngine()
    private let speechRecognizer: SFSpeechRecognizer?
    private var recognitionRequest: SFSpeechAudioBufferRecognitionRequest?
    private var recognitionTask: SFSpeechRecognitionTask?
    private var hasMicrophonePermission = false
    private var hasSpeechPermission = false

    /// Whether speech recognition is available on this device.
    var isAvailable: Bool {
        speechRecognizer?.isAvailable ?? false
    }

    /// Whether all permissions are granted.
    var hasPermissions: Bool {
        hasMicrophonePermission && hasSpeechPermission
    }

    init() {
        // Prefer Chinese, fall back to English
        self.speechRecognizer = SFSpeechRecognizer(locale: Locale(identifier: "zh-CN"))
            ?? SFSpeechRecognizer(locale: Locale(identifier: "en-US"))
            ?? SFSpeechRecognizer()
        checkPermissions()
    }

    // MARK: - Permissions

    private func checkPermissions() {
        // Speech recognition authorization
        switch SFSpeechRecognizer.authorizationStatus() {
        case .authorized:
            hasSpeechPermission = true
        case .notDetermined:
            SFSpeechRecognizer.requestAuthorization { [weak self] status in
                Task { @MainActor in
                    self?.hasSpeechPermission = status == .authorized
                }
            }
        default:
            hasSpeechPermission = false
        }

        // Microphone authorization (macOS)
        switch AVCaptureDevice.authorizationStatus(for: .audio) {
        case .authorized:
            hasMicrophonePermission = true
        case .notDetermined:
            AVCaptureDevice.requestAccess(for: .audio) { granted in
                Task { @MainActor [weak self] in
                    self?.hasMicrophonePermission = granted
                }
            }
        default:
            hasMicrophonePermission = false
        }
    }

    // MARK: - Control

    /// Toggle recording on/off.
    func toggle() {
        if isRecording {
            stop()
        } else {
            start()
        }
    }

    /// Start voice recording and recognition.
    func start() {
        guard !isRecording else { return }

        if !hasMicrophonePermission || !hasSpeechPermission {
            errorMessage = "Microphone and speech recognition permissions required"
            checkPermissions()
            return
        }

        guard let recognizer = speechRecognizer, recognizer.isAvailable else {
            errorMessage = "Speech recognizer is not available"
            return
        }

        // Reset transcribed text
        transcribedText = ""
        errorMessage = nil

        // Cancel any previous task
        recognitionTask?.cancel()
        recognitionTask = nil

        // Create recognition request
        recognitionRequest = SFSpeechAudioBufferRecognitionRequest()
        guard let recognitionRequest else {
            errorMessage = "Failed to create recognition request"
            return
        }
        recognitionRequest.shouldReportPartialResults = true

        // Start recognition
        recognitionTask = recognizer.recognitionTask(with: recognitionRequest) { [weak self] result, error in
            Task { @MainActor [weak self] in
                guard let self else { return }

                if let result {
                    self.transcribedText = result.bestTranscription.formattedString
                }

                if error != nil || (result?.isFinal ?? false) {
                    self.stop()
                }
            }
        }

        // Configure audio engine (macOS: no AVAudioSession needed)
        let inputNode = audioEngine.inputNode
        let recordingFormat = inputNode.outputFormat(forBus: 0)
        inputNode.installTap(onBus: 0, bufferSize: 1024, format: recordingFormat) { buffer, _ in
            recognitionRequest.append(buffer)
        }

        // Set flag before prepare/start so guard catches re-entrant calls
        isRecording = true

        audioEngine.prepare()
        do {
            try audioEngine.start()
            print("[VoiceInput] Recording started")
        } catch {
            isRecording = false
            errorMessage = "Failed to start microphone: \(error.localizedDescription)"
        }
    }

    /// Stop recording and finalize recognition.
    func stop() {
        guard isRecording else { return }

        // Set flag immediately to prevent re-entrant calls from recognition callback
        isRecording = false

        if audioEngine.isRunning {
            audioEngine.stop()
        }
        audioEngine.inputNode.removeTap(onBus: 0)

        recognitionRequest?.endAudio()
        recognitionRequest = nil
        recognitionTask?.cancel()
        recognitionTask = nil

        if !transcribedText.isEmpty {
            print("[VoiceInput] Final text: \(transcribedText)")
        }
    }
}
