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
                #if os(macOS)
                .frame(minWidth: 800, minHeight: 500)
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
