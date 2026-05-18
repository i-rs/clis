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
        // Log recognizer availability for debugging
        let cnRecognizer = SFSpeechRecognizer(locale: Locale(identifier: "zh-CN"))
        let enRecognizer = SFSpeechRecognizer(locale: Locale(identifier: "en-US"))
        print("[VoiceInput] zh-CN available: \(cnRecognizer?.isAvailable ?? false), en-US available: \(enRecognizer?.isAvailable ?? false)")

        self.speechRecognizer = cnRecognizer
            ?? enRecognizer
            ?? SFSpeechRecognizer()
        print("[VoiceInput] Using locale: \(self.speechRecognizer?.locale.identifier ?? "nil")")

        checkPermissions()
    }

    // MARK: - Permissions

    private func checkPermissions() {
        switch SFSpeechRecognizer.authorizationStatus() {
        case .authorized:
            hasSpeechPermission = true
            print("[VoiceInput] Speech recognition: authorized")
        case .notDetermined:
            print("[VoiceInput] Speech recognition: requesting authorization...")
            SFSpeechRecognizer.requestAuthorization { [weak self] status in
                Task { @MainActor [weak self] in
                    let granted = status == .authorized
                    print("[VoiceInput] Speech recognition: \(granted ? "granted" : "denied")")
                    self?.hasSpeechPermission = granted
                }
            }
        default:
            hasSpeechPermission = false
            print("[VoiceInput] Speech recognition: denied")
        }

        switch AVCaptureDevice.authorizationStatus(for: .audio) {
        case .authorized:
            hasMicrophonePermission = true
            print("[VoiceInput] Microphone: authorized")
        case .notDetermined:
            print("[VoiceInput] Microphone: requesting authorization...")
            AVCaptureDevice.requestAccess(for: .audio) { granted in
                Task { @MainActor [weak self] in
                    print("[VoiceInput] Microphone: \(granted ? "granted" : "denied")")
                    self?.hasMicrophonePermission = granted
                }
            }
        default:
            hasMicrophonePermission = false
            print("[VoiceInput] Microphone: denied")
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

        errorMessage = nil

        // Check permissions
        guard hasMicrophonePermission, hasSpeechPermission else {
            let msg = "Microphone and speech recognition permissions required"
            print("[VoiceInput] start: \(msg)")
            errorMessage = msg
            checkPermissions()
            return
        }

        // Check recognizer availability
        guard let recognizer = speechRecognizer else {
            errorMessage = "Speech recognizer not available on this device"
            print("[VoiceInput] start: no recognizer")
            return
        }
        guard recognizer.isAvailable else {
            errorMessage = "Speech recognizer is busy. Try again later."
            print("[VoiceInput] start: recognizer not available")
            return
        }

        // Check audio input availability
        let inputNode = audioEngine.inputNode
        let inputFormat = inputNode.outputFormat(forBus: 0)
        print("[VoiceInput] Audio input format: \(inputFormat.sampleRate)Hz, \(inputFormat.channelCount)ch")
        if inputFormat.sampleRate == 0 {
            errorMessage = "No audio input device found"
            print("[VoiceInput] start: no audio input device")
            return
        }

        // Reset
        transcribedText = ""

        // Cancel any previous task
        recognitionTask?.cancel()
        recognitionTask = nil

        // Create recognition request
        let request = SFSpeechAudioBufferRecognitionRequest()
        request.shouldReportPartialResults = true
        self.recognitionRequest = request

        // Start recognition task
        recognitionTask = recognizer.recognitionTask(with: request) { [weak self] result, error in
            guard let self else { return }

            if let error {
                // Log all errors — most common: "No speech detected", network error
                print("[VoiceInput] Recognition error: \(error.localizedDescription)")

                // Only propagate persistent errors, not transient ones
                if let sError = error as? NSError {
                    Task { @MainActor [weak self] in
                        switch sError.code {
                        case 203, 216: // No speech detected / recognition timed out
                            self?.errorMessage = "No speech detected. Please speak louder or check your microphone."
                        case 200: // Recognition error
                            self?.errorMessage = "Recognition failed: \(sError.localizedDescription)"
                        default:
                            self?.errorMessage = "Recognition error: \(sError.localizedDescription)"
                        }
                    }
                }
            }

            if let result {
                Task { @MainActor [weak self] in
                    self?.transcribedText = result.bestTranscription.formattedString
                }
            }

            // Stop only when recognition is truly finished
            if result?.isFinal == true {
                Task { @MainActor [weak self] in
                    self?.stop()
                }
            }
        }

        // Install audio tap — must use the node's actual output format
        inputNode.installTap(onBus: 0, bufferSize: 1024, format: inputFormat) { buffer, _ in
            request.append(buffer)
        }

        // Start audio engine
        isRecording = true
        audioEngine.prepare()
        do {
            try audioEngine.start()
            print("[VoiceInput] Recording started (format: \(inputFormat.sampleRate)Hz, \(inputFormat.channelCount)ch)")
        } catch {
            isRecording = false
            audioEngine.inputNode.removeTap(onBus: 0)
            errorMessage = "Failed to start microphone: \(error.localizedDescription)"
            print("[VoiceInput] Failed to start: \(error)")
        }
    }

    /// Stop recording and finalize recognition.
    func stop() {
        guard isRecording else { return }
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
            print("[VoiceInput] Final text (\(transcribedText.count) chars): \(transcribedText)")
        } else {
            print("[VoiceInput] No transcription result")
        }
    }
}
