import SwiftUI

struct ContentView: View {
    @EnvironmentObject var service: ClawService
    @EnvironmentObject var appState: AppState
    @State private var showingSettings = false
    @State private var showSessionList = false

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
            agentSection

            Section("Sessions") {
                ForEach(visibleSessions, id: \.id) { session in
                    SessionRow(session: session, isSelected: session.id == service.currentSession?.id)
                        .contentShape(Rectangle())
                        .onTapGesture {
                            appState.selectedTab = .sessions
                            service.switchToSession(session.id)
                        }
                        .swipeActions(edge: .trailing, allowsFullSwipe: true) {
                            Button(role: .destructive) {
                                service.deleteSession(session.id)
                            } label: {
                                Label("Delete", systemImage: "trash")
                            }
                        }
                }
            }

            Section("Manage") {
                ForEach(SidebarTab.allCases.filter { $0 != .sessions }) { tab in
                    ManageTabRow(
                        tab: tab,
                        isSelected: appState.selectedTab == tab
                    ) {
                        appState.selectedTab = tab
                    }
                }
            }
        }
        .listStyle(.sidebar)
        .navigationSplitViewColumnWidth(min: 220, ideal: 260, max: 380)
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

    private var visibleSessions: [ClawSession] {
        service.sessions.filter { session in
            (session.agentId == service.currentAgentId || session.agentId == nil) &&
            (appState.searchText.isEmpty || session.title.localizedCaseInsensitiveContains(appState.searchText))
        }
    }

    // MARK: - Agent Switcher

    @ViewBuilder
    private var agentSection: some View {
        Section {
            if service.agents.count <= 1 {
                singleAgentRow
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 8) {
                        ForEach(service.agents) { agent in
                            AgentChip(
                                agent: agent,
                                isActive: agent.id == service.currentAgentId
                            ) {
                                deferStateChange {
                                    Task { await service.switchAgent(agent.id) }
                                    appState.selectedTab = .sessions
                                }
                            }
                        }
                    }
                    .padding(.horizontal, 4)
                    .padding(.vertical, 2)
                }
            }
        } header: {
            Text("Agent")
        }
    }

    @ViewBuilder
    private var singleAgentRow: some View {
        HStack(spacing: 10) {
            ZStack {
                RoundedRectangle(cornerRadius: 8)
                    .fill(
                        LinearGradient(
                            colors: [.blue, .purple],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 28, height: 28)
                    .shadow(color: .blue.opacity(0.3), radius: 3, x: 0, y: 2)
                Image(systemName: "star.fill")
                    .font(.system(size: 12, weight: .semibold))
                    .foregroundStyle(.white)
            }
            Text(service.agents.first?.id ?? "default")
                .font(.callout)
                .fontWeight(.medium)
            Spacer()
        }
        .padding(.vertical, 6)
        .padding(.horizontal, 4)
        .background(
            RoundedRectangle(cornerRadius: 8, style: .continuous)
                .fill(Color.blue.opacity(0.08))
        )
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
            switch appState.selectedTab {
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

    private var searchPrompt: String {
        switch appState.selectedTab {
        case .sessions: "Search"
        case .tools: "Search tools"
        case .skills: "Search skills"
        case .plugins: "Search plugins"
        }
    }

    // MARK: - macOS / iPad (NavigationSplitView)

    private var splitBody: some View {
        NavigationSplitView {
            sidebarContent
        } detail: {
            detailContent
                .searchable(text: $appState.searchText, prompt: searchPrompt)
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
        .toolbar {
            ToolbarItem(placement: .automatic) {
                Button {
                    showingSettings = true
                } label: {
                    Image(systemName: "gearshape")
                }
                .help("Settings")
                .keyboardShortcut(",", modifiers: .command)
            }
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
        .searchable(text: $appState.searchText, prompt: "Search")
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

    // MARK: - Helpers

    private func deferStateChange(_ action: @escaping () -> Void) {
        DispatchQueue.main.async {
            action()
        }
    }
}

// MARK: - Agent Chip

struct AgentChip: View {
    let agent: ClawAgent
    let isActive: Bool
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            HStack(spacing: 6) {
                ZStack {
                    Circle()
                        .fill(isActive ? Color.white.opacity(0.3) : Color.white.opacity(0.15))
                        .frame(width: 20, height: 20)
                    Image(systemName: agent.id == "default" ? "star.fill" : "person.fill")
                        .font(.system(size: 9))
                        .foregroundStyle(.white)
                }

                Text(agent.id)
                    .font(.caption)
                    .fontWeight(isActive ? .semibold : .regular)
                    .foregroundStyle(isActive ? .white : .primary)
                    .lineLimit(1)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .background(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .fill(isActive ? Color.accentColor : Color.secondary.opacity(0.1))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .strokeBorder(isActive ? Color.clear : Color.secondary.opacity(0.2), lineWidth: 0.5)
            )
        }
        .buttonStyle(.plain)
    }
}

// MARK: - Manage Tab Row

struct ManageTabRow: View {
    let tab: SidebarTab
    let isSelected: Bool
    let action: () -> Void

    private var accent: Color {
        switch tab {
        case .tools: return .orange
        case .skills: return .green
        case .plugins: return .purple
        default: return .accentColor
        }
    }

    var body: some View {
        Button(action: action) {
            HStack(spacing: 10) {
                ZStack {
                    RoundedRectangle(cornerRadius: 6, style: .continuous)
                        .fill(accent.opacity(isSelected ? 0.2 : 0.1))
                        .frame(width: 28, height: 28)
                    Image(systemName: tab.icon)
                        .font(.system(size: 13, weight: .medium))
                        .foregroundStyle(accent)
                }

                Text(tab.label)
                    .font(.callout)
                    .fontWeight(isSelected ? .semibold : .regular)
                    .foregroundStyle(isSelected ? .primary : .secondary)

                Spacer()

                if isSelected {
                    Image(systemName: "chevron.right")
                        .font(.system(size: 10, weight: .semibold))
                        .foregroundStyle(accent)
                }
            }
            .padding(.vertical, 4)
        }
        .buttonStyle(.plain)
    }
}
