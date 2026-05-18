import SwiftUI

@main
struct IrsClawWatchApp: App {
    @StateObject private var service = ClawService()

    var body: some Scene {
        WindowGroup {
            WatchContentView()
                .environmentObject(service)
        }
    }
}

struct WatchContentView: View {
    @EnvironmentObject var service: ClawService
    @State private var inputText = ""
    @State private var isSending = false
    @State private var isMessageSent = false

    var body: some View {
        ScrollView {
            VStack(spacing: 10) {
                // Agent picker
                if !service.agents.isEmpty {
                    VStack(alignment: .leading, spacing: 4) {
                        Text("Agent")
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                        Picker("", selection: $service.currentAgentId) {
                            ForEach(service.agents) { agent in
                                Text(agent.id).tag(agent.id)
                            }
                        }
                        .pickerStyle(.navigationLink)
                        .onChange(of: service.currentAgentId) { _, newId in
                            if newId != service.currentSession?.agentId {
                                service.switchAgent(newId)
                            }
                        }
                    }
                }

                Divider()

                // Message input with built-in dictation
                VStack(spacing: 8) {
                    TextField("Type or dictate...", text: $inputText)
                        .disabled(isSending)

                    if !inputText.isEmpty {
                        Button("Send") {
                            sendMessage()
                        }
                        .buttonStyle(.borderedProminent)
                        .disabled(isSending)
                    }

                    if isSending {
                        ProgressView()
                            .scaleEffect(0.8)
                    }

                    if isMessageSent {
                        Text("Sent!")
                            .font(.caption2)
                            .foregroundStyle(.green)
                    }
                }
            }
            .padding()
        }
        .onAppear {
            if service.connectionState != .connected {
                service.connectToBackend()
            }
            Task { await service.fetchAgents() }
        }
    }

    private func sendMessage() {
        guard !inputText.isEmpty, !isSending else { return }
        isSending = true
        let text = inputText
        inputText = ""

        Task {
            service.sendMessage(text)
            try? await Task.sleep(nanoseconds: 3_000_000_000)
            isSending = false
            isMessageSent = true
            // Reset sent confirmation after a delay
            try? await Task.sleep(nanoseconds: 2_000_000_000)
            isMessageSent = false
        }
    }
}
