//
//  IrsClawApp.swift
//  IrsClaw
//
//  Created by mankong on 2026/5/19.
//

import SwiftUI

@main
struct IrsClawApp: App {
    @StateObject private var service = ClawService()

    var body: some Scene {
        WindowGroup {
            ContentView()
                .environmentObject(service)
                .frame(minWidth: 800, minHeight: 500)
                .onDisappear {
                    service.stopBackend()
                }
        }
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
    }
}
