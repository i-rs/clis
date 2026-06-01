import SwiftUI

struct ContentView: View {
    @EnvironmentObject var service: ClawService
    @EnvironmentObject var appState: AppState
    @State private var showingSettings = false

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
            case .usage:
                UsagePanel(service: service)
            case .agents:
                AgentsSettingsView(service: service)
            }
        }
    }

    private var searchPrompt: String {
        switch appState.selectedTab {
        case .sessions: return "Search"
        case .tools: return "Search tools"
        case .skills: return "Search skills"
        case .plugins: return "Search plugins"
        case .usage: return "Search"
        case .agents: return "Search agents"
        }
    }

    private var navigationTitle: String {
        switch appState.selectedTab {
        case .sessions: return service.currentSession?.title ?? "i-rs-claw"
        case .tools: return "Tools"
        case .skills: return "Skills"
        case .plugins: return "Plugins"
        case .usage: return "Token Usage"
        case .agents: return "Agents"
        }
    }

    // MARK: - iOS / iPad (NavigationSplitView)

    private var splitBody: some View {
        NavigationSplitView {
            sidebarContent
        } detail: {
            NavigationStack {
                detailContent
                    .searchable(text: $appState.searchText, prompt: searchPrompt)
            }
            .navigationTitle(navigationTitle)
            #if os(iOS)
            .navigationBarTitleDisplayMode(.large)
            #endif
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Button {
                        showingSettings = true
                    } label: {
                        Image(systemName: "gearshape")
                    }
                    .help("Settings")
                }
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

    // MARK: - iPhone (Sheet Menu)

    #if os(iOS)
    private var phoneBody: some View {
        NavigationStack(path: $appState.drawerPath) {
            ChatView(service: service)
                .navigationTitle("i-rs-claw")
                .navigationBarTitleDisplayMode(.inline)
                .toolbar {
                    ToolbarItem(placement: .navigationBarLeading) {
                        Button {
                            appState.showingDrawer = true
                        } label: {
                            Image(systemName: "line.3.horizontal")
                                .font(.system(size: 16, weight: .medium))
                        }
                    }

                    ToolbarItemGroup(placement: .navigationBarTrailing) {
                        newChatButton
                        settingsButton
                    }
                }
                .navigationDestination(for: DrawerDestination.self) { destination in
                    switch destination {
                    case .sessions:
                        SessionListView(service: service)
                            .navigationTitle("Sessions")
                    case .tools:
                        ToolsPanel(service: service)
                            .navigationTitle("Tools")
                    case .skills:
                        SkillsPanel(service: service)
                            .navigationTitle("Skills")
                    case .plugins:
                        PluginsPanel(service: service)
                            .navigationTitle("Plugins")
                    case .usage:
                        UsagePanel(service: service)
                            .navigationTitle("Token Usage")
                    case .agents:
                        AgentsSettingsView(service: service)
                            .navigationTitle("Agents")
                    }
                }
        }
        .sheet(isPresented: $appState.showingDrawer) {
            DrawerMenuView(service: service, appState: appState)
                .presentationDetents([.medium, .large])
                .presentationDragIndicator(.visible)
        }
        .sheet(isPresented: $showingSettings) {
            SettingsView(service: service)
                .presentationDetents([.medium, .large])
                .presentationDragIndicator(.visible)
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

    private var newChatButton: some View {
        Button {
            Task { await service.createSession() }
        } label: {
            Image(systemName: "square.and.pencil")
                .font(.system(size: 15, weight: .medium))
        }
        .disabled(service.connectionState != .connected)
    }

    private var settingsButton: some View {
        Button {
            showingSettings = true
        } label: {
            Image(systemName: "gearshape")
                .font(.system(size: 15, weight: .medium))
        }
    }
    #endif

    private var sessionPicker: some View {
        ForEach(service.sessions) { session in
            Button(session.title) {
                service.switchToSession(session.id)
            }
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

// MARK: - Drawer Menu View

#if os(iOS)
struct DrawerMenuView: View {
    @ObservedObject var service: ClawService
    @ObservedObject var appState: AppState
    @Environment(\.dismiss) private var dismiss

    private var visibleSessions: [ClawSession] {
        service.sessions.filter { session in
            (session.agentId == service.currentAgentId || session.agentId == nil) &&
            (appState.searchText.isEmpty || session.title.localizedCaseInsensitiveContains(appState.searchText))
        }
    }

    var body: some View {
        NavigationStack {
            ScrollView {
                VStack(spacing: 24) {
                    // Agent Card
                    agentCard

                    // Navigation Grid
                    navigationGrid

                    // Recent Sessions
                    if !visibleSessions.isEmpty {
                        recentSessionsSection
                    }
                }
                .padding()
            }
            .navigationTitle("Menu")
            .navigationBarTitleDisplayMode(.large)
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("Done") {
                        dismiss()
                    }
                    .fontWeight(.semibold)
                }
            }
        }
    }

    // MARK: - Agent Card

    private var agentCard: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Agent")
                .font(.subheadline)
                .fontWeight(.semibold)
                .foregroundStyle(.secondary)

            if service.agents.count <= 1 {
                singleAgentRow
            } else {
                ScrollView(.horizontal, showsIndicators: false) {
                    HStack(spacing: 10) {
                        ForEach(service.agents) { agent in
                            AgentChip(
                                agent: agent,
                                isActive: agent.id == service.currentAgentId
                            ) {
                                Task { await service.switchAgent(agent.id) }
                            }
                        }
                    }
                }
            }
        }
        .padding()
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color(uiColor: .secondarySystemGroupedBackground))
        .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
    }

    @ViewBuilder
    private var singleAgentRow: some View {
        HStack(spacing: 12) {
            ZStack {
                Circle()
                    .fill(
                        LinearGradient(
                            colors: [.blue, .purple],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 36, height: 36)
                Image(systemName: "star.fill")
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundStyle(.white)
            }

            Text(service.agents.first?.id ?? "default")
                .font(.body)
                .fontWeight(.medium)
        }
    }

    // MARK: - Navigation Grid

    private var navigationGrid: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Browse")
                .font(.subheadline)
                .fontWeight(.semibold)
                .foregroundStyle(.secondary)

            LazyVGrid(columns: [GridItem(.flexible()), GridItem(.flexible())], spacing: 12) {
                navigationCard(
                    icon: "message.fill",
                    title: "Sessions",
                    color: .blue
                ) {
                    appState.selectedTab = .sessions
                    appState.drawerPath = [.sessions]
                    dismiss()
                }

                navigationCard(
                    icon: "wrench.and.screwdriver.fill",
                    title: "Tools",
                    color: .orange
                ) {
                    appState.selectedTab = .tools
                    appState.drawerPath = [.tools]
                    dismiss()
                }

                navigationCard(
                    icon: "book.fill",
                    title: "Skills",
                    color: .green
                ) {
                    appState.selectedTab = .skills
                    appState.drawerPath = [.skills]
                    dismiss()
                }

                navigationCard(
                    icon: "puzzlepiece.extension.fill",
                    title: "Plugins",
                    color: .purple
                ) {
                    appState.selectedTab = .plugins
                    appState.drawerPath = [.plugins]
                    dismiss()
                }

                navigationCard(
                    icon: "chart.bar.fill",
                    title: "Usage",
                    color: .blue
                ) {
                    appState.selectedTab = .usage
                    appState.drawerPath = [.usage]
                    dismiss()
                }

                navigationCard(
                    icon: "person.2.fill",
                    title: "Agents",
                    color: .teal
                ) {
                    appState.selectedTab = .agents
                    appState.drawerPath = [.agents]
                    dismiss()
                }
            }
        }
    }

    private func navigationCard(icon: String, title: String, color: Color, action: @escaping () -> Void) -> some View {
        Button(action: action) {
            VStack(spacing: 10) {
                ZStack {
                    RoundedRectangle(cornerRadius: 12, style: .continuous)
                        .fill(color.opacity(0.15))
                        .frame(width: 48, height: 48)

                    Image(systemName: icon)
                        .font(.system(size: 20, weight: .medium))
                        .foregroundStyle(color)
                }

                Text(title)
                    .font(.subheadline)
                    .fontWeight(.medium)
                    .foregroundStyle(.primary)
            }
            .frame(maxWidth: .infinity)
            .padding(.vertical, 16)
            .background(Color(uiColor: .secondarySystemGroupedBackground))
            .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        }
        .buttonStyle(.plain)
    }

    // MARK: - Recent Sessions

    private var recentSessionsSection: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Recent Sessions")
                    .font(.subheadline)
                    .fontWeight(.semibold)
                    .foregroundStyle(.secondary)

                Spacer()

                Button("See All") {
                    appState.selectedTab = .sessions
                    appState.drawerPath = [.sessions]
                    dismiss()
                }
                .font(.subheadline)
            }

            VStack(spacing: 0) {
                ForEach(visibleSessions.prefix(5), id: \.id) { session in
                    Button {
                        service.switchToSession(session.id)
                        dismiss()
                    } label: {
                        HStack(spacing: 12) {
                            Image(systemName: "message.fill")
                                .font(.system(size: 14))
                                .foregroundStyle(.secondary)
                                .frame(width: 24)

                            Text(session.title)
                                .font(.body)
                                .lineLimit(1)

                            Spacer()

                            Text(session.shortDate)
                                .font(.caption)
                                .foregroundStyle(.tertiary)
                        }
                        .padding()
                        .contentShape(Rectangle())
                    }
                    .buttonStyle(.plain)

                    if session.id != visibleSessions.prefix(5).last?.id {
                        Divider()
                            .padding(.leading, 56)
                    }
                }
            }
            .background(Color(uiColor: .secondarySystemGroupedBackground))
            .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        }
    }
}
#endif

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
        case .usage: return .blue
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
