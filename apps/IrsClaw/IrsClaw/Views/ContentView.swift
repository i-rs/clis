import SwiftUI

struct ContentView: View {
    @EnvironmentObject var service: ClawService
    @State private var showingSettings = false
    @State private var searchText = ""
    @State private var showSessionList = false

    var filteredSessions: [ClawSession] {
        guard !searchText.isEmpty else { return service.sessions }
        return service.sessions.filter { session in
            session.title.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        #if os(iOS)
        if UIDevice.current.userInterfaceIdiom == .pad {
            splitBody
        } else {
            phoneBody
        }
        #else
        macBody
        #endif
    }

    // MARK: - Shared detail content

    @ViewBuilder
    private var detailContent: some View {
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

    // MARK: - macOS / iPad (NavigationSplitView)

    private var splitBody: some View {
        NavigationSplitView {
            SessionListView(
                service: service,
                sessions: filteredSessions,
                searchText: $searchText
            )
        } detail: {
            detailContent
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

    // MARK: - macOS only wrapper

    #if os(macOS)
    private var macBody: some View {
        splitBody
    }
    #endif

    // MARK: - iPhone (NavigationStack)

    #if os(iOS)
    private var phoneBody: some View {
        NavigationStack {
            if service.currentSession != nil && service.connectionState.isConnected {
                ChatView(service: service)
            } else {
                VStack(spacing: 0) {
                    detailContent
                }
            }
        }
        .navigationTitle("i-rs-claw")
        .navigationBarTitleDisplayMode(.inline)
        .toolbar { toolbarContent }
        .toolbarTitleMenu {
            sessionPicker
        }
        .sheet(isPresented: $showSessionList) {
            NavigationStack {
                sessionListView
                    .navigationTitle("Sessions")
                    .navigationBarTitleDisplayMode(.inline)
                    .toolbar {
                        ToolbarItem(placement: .confirmationAction) {
                            Button("Done") { showSessionList = false }
                        }
                    }
            }
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

    @ToolbarContentBuilder
    private var toolbarContent: some ToolbarContent {
        ToolbarItemGroup(placement: .navigationBarTrailing) {
            if service.isProcessing {
                ProgressView()
                    .scaleEffect(0.7)
            }

            Button {
                Task { await service.createSession() }
            } label: {
                Image(systemName: "square.and.pencil")
            }
            .disabled(service.connectionState != .connected)

            Button {
                showingSettings = true
            } label: {
                Image(systemName: "gearshape")
            }
        }

        ToolbarItem(placement: .navigationBarLeading) {
            Button {
                showSessionList = true
            } label: {
                Image(systemName: "list.bullet")
            }
            .disabled(service.connectionState != .connected)
        }
    }

    private var sessionPicker: some View {
        ForEach(service.sessions) { session in
            Button(session.title) {
                service.switchToSession(session.id)
            }
        }
    }

    private var sessionListView: some View {
        List {
            if service.sessions.isEmpty {
                ContentUnavailableView(
                    "No Sessions",
                    systemImage: "text.bubble",
                    description: Text("Tap + to create a new chat")
                )
            } else {
                ForEach(service.sessions) { session in
                    SessionRow(session: session)
                        .contentShape(Rectangle())
                        .onTapGesture {
                            service.switchToSession(session.id)
                            showSessionList = false
                        }
                        .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                            Button("Delete", role: .destructive) {
                                service.deleteSession(session.id)
                            }
                        }
                }
            }
        }
        .searchable(text: $searchText, prompt: "Search")
    }
    #endif

    // MARK: - Detail Views

    @ViewBuilder
    private var disconnectedView: some View {
        VStack(spacing: 16) {
            Image(systemName: "bolt.horizontal.circle")
                .font(.system(size: 48))
                .foregroundStyle(.secondary)
            Text("Connect to i-rs-claw Backend")
                .font(.title2)
                .multilineTextAlignment(.center)
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
