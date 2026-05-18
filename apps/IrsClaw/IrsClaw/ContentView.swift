//
//  ContentView.swift
//  IrsClaw
//
//  Created by mankong on 2026/5/19.
//

import SwiftUI

struct ContentView: View {
    @EnvironmentObject var service: ClawService
    @State private var showingSettings = false
    @State private var searchText = ""

    var filteredSessions: [ClawSession] {
        guard !searchText.isEmpty else { return service.sessions }
        return service.sessions.filter { session in
            session.title.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        NavigationSplitView {
            SessionListView(
                service: service,
                sessions: filteredSessions,
                searchText: $searchText
            )
        } detail: {
            switch service.connectionState {
            case .disconnected, .failed:
                disconnectedView
            case .waitingForHealth:
                connectingView
            case .connected:
                if service.currentSession != nil {
                    ChatView(service: service)
                } else {
                    emptySessionView
                }
            }
        }
        .toolbar {
            ToolbarItemGroup {
                if service.isProcessing {
                    ProgressView()
                        .scaleEffect(0.7)
                        .help("Processing...")
                }

                Button {
                    Task { await service.createSession() }
                } label: {
                    Label("New Chat", systemImage: "square.and.pencil")
                }
                .help("New Chat")
                .disabled(service.connectionState != .connected)

                Button {
                    showingSettings = true
                } label: {
                    Label("Settings", systemImage: "gearshape")
                }
                .help("Settings")
            }
        }
        .sheet(isPresented: $showingSettings) {
            SettingsView(service: service)
        }
        .onAppear {
            if service.connectionState != .connected {
                service.connectToBackend()
            }
        }
        .onDisappear {
            service.stopBackend()
        }
    }

    // MARK: - Detail Views

    @ViewBuilder
    private var disconnectedView: some View {
        VStack(spacing: 16) {
            Image(systemName: "bolt.horizontal.circle")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("Connect to i-rs-claw Backend")
                .font(.title2)
            if let error = service.connectionState.errorMessage {
                Text(error)
                    .foregroundStyle(.red)
                    .font(.callout)
                    .multilineTextAlignment(.center)
                    .padding(.horizontal)
            }

            VStack(spacing: 8) {
                Button("Connect to Backend") {
                    service.connectToBackend()
                }
                .buttonStyle(.borderedProminent)
                .disabled(service.connectionState.isConnecting)
            }
        }
        .padding()
    }

    private var connectingView: some View {
        VStack(spacing: 16) {
            ProgressView()
                .scaleEffect(1.2)
            Text("Connecting to i-rs-claw backend...")
                .font(.headline)
                .foregroundStyle(.secondary)
            Text("Please make sure the dashboard is running")
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
        .padding()
    }

    private var emptySessionView: some View {
        VStack(spacing: 12) {
            Image(systemName: "message")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text("Select or create a session to begin")
                .foregroundStyle(.secondary)
            Button("New Chat") {
                Task { await service.createSession() }
            }
            .buttonStyle(.borderedProminent)
        }
    }
}
