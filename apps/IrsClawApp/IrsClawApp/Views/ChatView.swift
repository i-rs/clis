import SwiftUI

struct ChatView: View {
    @ObservedObject var service: ClawService
    @State private var inputText = ""
    @State private var scrollToBottom = false
    @State private var showingAddAgent = false
    @FocusState private var isInputFocused: Bool
    @StateObject private var voiceInput = VoiceInputService()

    var body: some View {
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
            .scrollDismissesKeyboard(.immediately)
            .onTapGesture { isInputFocused = false }
            .onChange(of: service.messageVersion) { _, _ in
                withAnimation(.easeOut(duration: 0.15)) {
                    proxy.scrollTo("bottom", anchor: .bottom)
                }
            }
        }
        .safeAreaInset(edge: .bottom, spacing: 0) {
            VStack(spacing: 0) {
                if voiceInput.isRecording || !(voiceInput.errorMessage?.isEmpty ?? true) {
                    VStack(spacing: 0) {
                        if voiceInput.isRecording {
                            recordingBar
                                .transition(.move(edge: .bottom).combined(with: .opacity))
                        } else if let error = voiceInput.errorMessage, !error.isEmpty {
                            errorBar(error)
                                .transition(.move(edge: .bottom).combined(with: .opacity))
                        }
                    }
                    .padding(.horizontal, 12)
                    .padding(.bottom, 6)
                }

                floatingInputBar
                    .padding(.bottom, 8)
                    .background(Color.platformWindowBackground)
            }
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

    // MARK: - ⭐ Floating Input Bar

    @ViewBuilder
    private var floatingInputBar: some View {
        HStack(spacing: 8) {
            micButton

            TextField("Message i-rs-claw...", text: $inputText, axis: .vertical)
                .lineLimit(1...5)
                .textFieldStyle(.plain)
                .font(.body)
                .focused($isInputFocused)
                .toolbar {
                    ToolbarItemGroup(placement: .keyboard) {
                        Spacer()
                        Button("Done") { isInputFocused = false }
                            .fontWeight(.semibold)
                    }
                }

            sendButton
        }
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
        .background(
            RoundedRectangle(cornerRadius: 28, style: .continuous)
                .fill(.ultraThinMaterial)
                .shadow(color: .black.opacity(0.15), radius: 12, x: 0, y: 4)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 28, style: .continuous)
                .strokeBorder(Color.white.opacity(0.1), lineWidth: 0.5)
        )
        .padding(.horizontal, 12)
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
                    .fill(Color.platformSecondaryBackground)
                    .frame(width: 38, height: 38)

                Image(systemName: voiceInput.isRecording ? "mic.fill" : "mic")
                    .font(.system(size: 15, weight: .medium))
                    .foregroundStyle(voiceInput.isRecording ? .red : .secondary)
            }
        }
        .buttonStyle(.plain)
        .help("Voice Input")
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
                    .fill(hasContent ? Color.accentColor : Color.platformSecondaryBackground)
                    .frame(width: 38, height: 38)

                Image(systemName: "arrow.up")
                    .font(.system(size: 15, weight: .semibold))
                    .foregroundStyle(hasContent ? .white : .secondary)
            }
        }
        .buttonStyle(.plain)
        .disabled(!hasContent || service.isProcessing)
        .keyboardShortcut(.return, modifiers: .command)
        .animation(.spring(response: 0.3, dampingFraction: 0.75), value: hasContent)
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
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
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
        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
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
