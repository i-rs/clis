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
            HStack(spacing: 6) {
                TextField("Ask i-rs-claw...", text: $inputText)
                    .textFieldStyle(.plain)
                    .padding(8)
                    .background(Color.platformControlBackground)
                    .clipShape(RoundedRectangle(cornerRadius: 8))
                    .disabled(service.isProcessing)

                // Microphone button
                Button {
                    voiceInput.toggle()
                    if !voiceInput.isRecording, !voiceInput.transcribedText.isEmpty {
                        inputText = voiceInput.transcribedText
                    }
                } label: {
                    Image(systemName: voiceInput.isRecording
                          ? "mic.fill"
                          : "mic")
                        .font(.title3)
                        .foregroundStyle(voiceInput.isRecording ? .red : .secondary)
                }
                .buttonStyle(.plain)
                .help("Voice Input (⌥V)")
                .keyboardShortcut("v", modifiers: .option)
                .disabled(!voiceInput.isAvailable || service.isProcessing)

                // Send button
                let hasContent = !inputText.trimmingCharacters(in: .whitespaces).isEmpty
                Button {
                    sendMessage()
                } label: {
                    Image(systemName: hasContent ? "arrow.up.circle.fill" : "arrow.up.circle")
                        .font(.title2)
                        .foregroundColor(hasContent ? .accentColor : .secondary)
                        .scaleEffect(hasContent ? 1.1 : 1.0)
                        .animation(.spring(response: 0.2, dampingFraction: 0.6), value: hasContent)
                }
                .buttonStyle(.plain)
                .disabled(!hasContent || service.isProcessing)
                .keyboardShortcut(.return, modifiers: .command)
            }
            .padding(12)
            .background(Color.platformWindowBackground)
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

    // MARK: - Recording Bar

    @ViewBuilder
    private var recordingBar: some View {
        HStack(spacing: 6) {
            Circle()
                .fill(.red)
                .frame(width: 6, height: 6)
                .opacity(0.8)

            Text("Recording...")
                .font(.caption)
                .foregroundStyle(.secondary)

            if !voiceInput.transcribedText.isEmpty {
                Text(voiceInput.transcribedText)
                    .font(.caption)
                    .foregroundStyle(.tertiary)
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
        .padding(.horizontal, 12)
        .padding(.vertical, 6)
        .background(Color.red.opacity(0.05))
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
