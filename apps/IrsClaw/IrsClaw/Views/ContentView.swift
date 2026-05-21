import SwiftUI

struct ContentView: View {
    @EnvironmentObject var service: ClawService
    @State private var showingSettings = false
    @State private var searchText = ""
    @State private var showSessionList = false
    @State private var selectedTab: SidebarTab = .sessions

    enum SidebarTab: String, CaseIterable, Identifiable {
        case sessions, tools, skills, plugins

        var id: String { rawValue }

        var label: String {
            switch self {
            case .sessions: return "Sessions"
            case .tools: return "Tools"
            case .skills: return "Skills"
            case .plugins: return "Plugins"
            }
        }

        var icon: String {
            switch self {
            case .sessions: return "message"
            case .tools: return "wrench.adjustable"
            case .skills: return "book"
            case .plugins: return "puzzlepiece"
            }
        }

        var count: Int { 0 }
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

    // MARK: - Sidebar

    @ViewBuilder
    private var sidebarContent: some View {
        List {
            agentSwitcherSection

            Section("Chat") {
                ForEach(SidebarTab.allCases.filter { $0 != .sessions }) { tab in
                    Label(tab.label, systemImage: tab.icon)
                        .contentShape(Rectangle())
                        .onTapGesture {
                            Task { @MainActor in
                                selectedTab = tab
                            }
                        }
                }
            }

            Section("Sessions") {
                ForEach(service.sessions, id: \.id) { session in
                    let isMatchingAgent = session.agentId == service.currentAgentId || session.agentId == nil
                    let isMatchingSearch = searchText.isEmpty || session.title.localizedCaseInsensitiveContains(searchText)
                    if isMatchingAgent && isMatchingSearch {
                        SessionRow(session: session)
                            .contentShape(Rectangle())
                            .onTapGesture {
                                selectedTab = .sessions
                                Task { @MainActor in
                                    service.switchToSession(session.id)
                                }
                            }
                    }
                }
            }
        }
        .listStyle(.sidebar)
        .searchable(text: $searchText, prompt: "Search")
        .navigationSplitViewColumnWidth(220)
        .toolbar {
            ToolbarItem(placement: .primaryAction) {
                Button {
                    Task { await service.createSession() }
                } label: {
                    Image(systemName: "square.and.pencil")
                }
                .help("New Chat")
                .disabled(service.connectionState != .connected)
            }
        }
    }

    @ViewBuilder
    private var agentSwitcherSection: some View {
        Section {
            Picker("", selection: Binding(
                get: {
                    service.agents.first(where: { $0.id == service.currentAgentId })
                        .map { $0.id } ?? "default"
                },
                set: { newId in
                    guard newId != service.currentAgentId else { return }
                    Task { @MainActor in
                        await service.switchAgent(newId)
                        selectedTab = .sessions
                    }
                }
            )) {
                ForEach(service.agents, id: \.id) { agent in
                    Text(agent.id).tag(agent.id)
                }
            }
            .pickerStyle(.menu)
            .labelsHidden()
            .font(.body)
            .disabled(service.agents.isEmpty)
        } header: {
            Text("Agent")
        }
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
            switch selectedTab {
            case .sessions:
                if service.currentSession != nil {
                    ChatView(service: service)
                } else {
                    emptySessionView
                }
            case .tools:
                ToolsPanel(service: service)
            case .skills:
                SkillsPanel(service: service)
            case .plugins:
                PluginsPanel(service: service)
            }
        }
    }

    // MARK: - macOS / iPad (NavigationSplitView)

    private var splitBody: some View {
        NavigationSplitView {
            sidebarContent
        } detail: {
            detailContent
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
