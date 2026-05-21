import SwiftUI

struct ChatView: View {
    @ObservedObject var service: ClawService
    @State private var inputText = ""
    @State private var scrollToBottom = false
    @State private var showingAddAgent = false
    @StateObject private var voiceInput = VoiceInputService()

    var body: some View {
        VStack(spacing: 0) {
            // Message List
            ScrollViewReader { proxy in
                ScrollView {
                    LazyVStack(spacing: 6) {
                        ForEach(service.messages) { item in
                            MessageBubbleView(message: item.message)
                                .id(item.id)
                        }

                        Color.clear
                            .frame(height: 1)
                            .id("bottom")
                    }
                    .padding(.horizontal, 16)
                    .padding(.vertical, 8)
                }
                .onChange(of: service.messageVersion) { _, _ in
                    withAnimation(.easeOut(duration: 0.15)) {
                        proxy.scrollTo("bottom", anchor: .bottom)
                    }
                }
            }

            Divider()

            // Recording / error indicator bar
            if voiceInput.isRecording || !(voiceInput.errorMessage?.isEmpty ?? true) {
                if voiceInput.isRecording {
                    recordingBar
                } else if let error = voiceInput.errorMessage, !error.isEmpty {
                    errorBar(error)
                }
            }

            // Input Bar
            HStack(spacing: 8) {
                TextField("Ask i-rs-claw...", text: $inputText, axis: .vertical)
                    .lineLimit(1...5)
                    .textFieldStyle(.plain)
                    .font(.body)
                    .padding(.horizontal, 14)
                    .padding(.vertical, 10)
                    .background(
                        RoundedRectangle(cornerRadius: 20, style: .continuous)
                            .fill(Color.platformControlBackground)
                            .shadow(color: .black.opacity(0.04), radius: 2, x: 0, y: 1)
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 20, style: .continuous)
                            .strokeBorder(Color.secondary.opacity(0.12), lineWidth: 0.5)
                    )
                    .disabled(service.isProcessing)

                micButton

                sendButton
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            .background(.bar)
        }
        .onChange(of: voiceInput.transcribedText) { _, newText in
            if voiceInput.isRecording {
                inputText = newText
            }
        }
        .onChange(of: voiceInput.isRecording) { _, isNowRecording in
            if !isNowRecording, !voiceInput.transcribedText.isEmpty {
                inputText = voiceInput.transcribedText
            }
        }
        .onChange(of: voiceInput.errorMessage) { _, error in
            if let error, !error.isEmpty {
                print("[ChatView] Voice error: \(error)")
            }
        }
        .sheet(isPresented: $showingAddAgent) {
            AddAgentSheet(service: service)
        }
    }

    @ViewBuilder
    private var micButton: some View {
        Button {
            voiceInput.toggle()
            if !voiceInput.isRecording, !voiceInput.transcribedText.isEmpty {
                inputText = voiceInput.transcribedText
            }
        } label: {
            ZStack {
                Circle()
                    .fill(voiceInput.isRecording
                          ? Color.red.opacity(0.15)
                          : Color.secondary.opacity(0.08))
                    .frame(width: 36, height: 36)
                if voiceInput.isRecording {
                    Circle()
                        .stroke(Color.red.opacity(0.3), lineWidth: 2)
                        .frame(width: 32, height: 32)
                }
                Image(systemName: voiceInput.isRecording ? "mic.fill" : "mic")
                    .font(.system(size: 14, weight: voiceInput.isRecording ? .semibold : .regular))
                    .foregroundStyle(voiceInput.isRecording ? .red : .secondary)
            }
        }
        .buttonStyle(.plain)
        .help("Voice Input (⌥V)")
        .keyboardShortcut("v", modifiers: .option)
        .disabled(!voiceInput.isAvailable || service.isProcessing)
    }

    @ViewBuilder
    private var sendButton: some View {
        let hasContent = !inputText.trimmingCharacters(in: .whitespaces).isEmpty
        Button {
            sendMessage()
        } label: {
            ZStack {
                Circle()
                    .fill(hasContent ? Color.accentColor : Color.clear)
                    .frame(width: 36, height: 36)
                Image(systemName: "arrow.up")
                    .font(.system(size: 14, weight: .bold))
                    .foregroundStyle(hasContent ? .white : .secondary)
            }
            .overlay(
                Circle()
                    .strokeBorder(hasContent ? Color.clear : Color.secondary.opacity(0.2), lineWidth: 1)
                    .frame(width: 36, height: 36)
            )
        }
        .buttonStyle(.plain)
        .disabled(!hasContent || service.isProcessing)
        .keyboardShortcut(.return, modifiers: .command)
        .animation(.spring(response: 0.25, dampingFraction: 0.7), value: hasContent)
    }

    // MARK: - Recording Bar

    @ViewBuilder
    private var recordingBar: some View {
        HStack(spacing: 8) {
            PulsingDot()

            Text("Listening")
                .font(.caption)
                .fontWeight(.medium)
                .foregroundStyle(.red.opacity(0.8))

            if !voiceInput.transcribedText.isEmpty {
                Text(voiceInput.transcribedText)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
                    .truncationMode(.tail)
            }

            Spacer()

            Button("Done") {
                voiceInput.stop()
                if !voiceInput.transcribedText.isEmpty {
                    inputText = voiceInput.transcribedText
                }
            }
            .controlSize(.small)
            .buttonStyle(.borderedProminent)
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 8)
        .background(.red.opacity(0.04))
        .transition(.move(edge: .bottom).combined(with: .opacity))
    }

    // MARK: - Error Bar

    @ViewBuilder
    private func errorBar(_ error: String) -> some View {
        HStack(spacing: 6) {
            Image(systemName: "exclamationmark.microphone")
                .foregroundStyle(.orange)
                .font(.caption)

            Text(error)
                .font(.caption)
                .foregroundStyle(.secondary)

            Spacer()

            Button("Dismiss") {
                voiceInput.errorMessage = nil
            }
            .controlSize(.small)
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color.orange.opacity(0.05))
        .transition(.move(edge: .bottom).combined(with: .opacity))
    }

    // MARK: - Actions

    private func sendMessage() {
        let text = inputText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !text.isEmpty else { return }
        inputText = ""
        service.sendMessage(text)
    }
}

struct PulsingDot: View {
    @State private var isPulsing = false

    var body: some View {
        ZStack {
            Circle()
                .fill(.red.opacity(0.2))
                .frame(width: 14, height: 14)
                .scaleEffect(isPulsing ? 1.4 : 1.0)
            Circle()
                .fill(.red)
                .frame(width: 6, height: 6)
        }
        .onAppear {
            withAnimation(.easeInOut(duration: 0.8).repeatForever(autoreverses: true)) {
                isPulsing = true
            }
        }
    }
}
