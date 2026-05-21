import SwiftUI

@main
struct IrsClawApp: App {
    @StateObject private var service = ClawService()
    @StateObject private var appState = AppState()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(service)
                .environmentObject(appState)
                #if os(macOS)
                .frame(minWidth: 960, idealWidth: 1100, minHeight: 620, idealHeight: 720)
                #endif
                .onDisappear {
                    service.stopBackend()
                }
        }
        #if os(macOS)
        .windowResizability(.contentMinSize)
        .commands {
            CommandGroup(after: .newItem) {
                Button("New Chat") {
                    Task { await service.createSession() }
                }
                .keyboardShortcut("n", modifiers: .command)
                .disabled(!service.connectionState.isConnected)
            }
        }
        #endif
    }
}
