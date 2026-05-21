import SwiftUI

// MARK: - Settings Sections

enum SettingsSection: String, CaseIterable, Identifiable {
    case general, agents, backend

    var id: String { rawValue }

    var label: String {
        switch self {
        case .general: return "General"
        case .agents: return "Agents"
        case .backend: return "Backend"
        }
    }

    var icon: String {
        switch self {
        case .general: return "gearshape"
        case .agents: return "person.2"
        case .backend: return "bolt"
        }
    }

    var subtitle: String {
        switch self {
        case .general: return "Connection, LLM config"
        case .agents: return "Manage agent profiles"
        case .backend: return "API, sessions, errors"
        }
    }
}

// MARK: - Main Settings View

struct SettingsView: View {
    @ObservedObject var service: ClawService
    @Environment(\.dismiss) private var dismiss
    @State private var selectedSection: SettingsSection? = .general

    var body: some View {
        NavigationSplitView {
            List(selection: $selectedSection) {
                ForEach(SettingsSection.allCases) { section in
                    Label {
                        VStack(alignment: .leading, spacing: 1) {
                            Text(section.label)
                                .font(.body)
                            Text(section.subtitle)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                        }
                    } icon: {
                        Image(systemName: section.icon)
                            .foregroundStyle(.tint)
                    }
                    .padding(.vertical, 1)
                    .tag(section)
                }
            }
            .listStyle(.sidebar)
            .navigationSplitViewColumnWidth(220)
        } detail: {
            detailView
                .padding(.horizontal)
        }
        #if os(macOS)
        .frame(width: 620, height: 460)
        #endif
    }

    @ViewBuilder
    private var detailView: some View {
        switch selectedSection ?? .general {
        case .general: GeneralSettingsView(service: service)
        case .agents: AgentsSettingsView(service: service)
        case .backend: BackendSettingsView(service: service)
        }
    }
}

// MARK: - General Settings

struct GeneralSettingsView: View {
    @ObservedObject var service: ClawService

    var body: some View {
        Form {
            // Connection Status Card
            Section {
                HStack(spacing: 12) {
                    ZStack {
                        RoundedRectangle(cornerRadius: 8)
                            .fill(service.connectionState.isConnected
                                  ? Color.green.opacity(0.12)
                                  : Color.red.opacity(0.12))
                            .frame(width: 36, height: 36)
                        Image(systemName: service.connectionState.isConnected
                              ? "antenna.radiowaves.left.and.right"
                              : "antenna.radiowaves.left.and.right.slash")
                            .foregroundStyle(service.connectionState.isConnected ? .green : .red)
                            .font(.title3)
                    }

                    VStack(alignment: .leading, spacing: 2) {
                        Text("Backend Connection")
                            .font(.body)
                        Text(service.connectionState.isConnected
                             ? "Connected to i-rs-claw dashboard"
                             : service.connectionState.errorMessage ?? "Disconnected")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    Circle()
                        .fill(service.connectionState.isConnected ? Color.green : Color.red)
                        .frame(width: 10, height: 10)
                }
                .padding(.vertical, 4)
            }

            // LLM Configuration
            if let config = service.config {
                Section {
                    SettingsRow(icon: "cube", iconColor: .blue) {
                        LabeledContent("Provider", value: config.provider ?? "—")
                    }
                    SettingsRow(icon: "cpu", iconColor: .purple) {
                        LabeledContent("Model", value: config.model ?? "—")
                    }
                    if let tools = config.enabledTools {
                        SettingsRow(icon: "wrench.adjustable", iconColor: .orange) {
                            LabeledContent("Tools") {
                                Text(tools.isEmpty ? "All enabled" : "\(tools.count) enabled")
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                    if let servers = config.mcpServers, !servers.isEmpty {
                        SettingsRow(icon: "server.rack", iconColor: .indigo) {
                            LabeledContent("MCP Servers") {
                                Text("\(servers.count) configured")
                                    .foregroundStyle(.secondary)
                            }
                        }
                    }
                } header: {
                    Label("LLM Configuration", systemImage: "brain")
                }
            }

            // Data section
            Section {
                SettingsRow(icon: "clock", iconColor: .gray) {
                    LabeledContent("Last Updated") {
                        Text("Just now")
                            .foregroundStyle(.secondary)
                    }
                }
            } header: {
                Label("Data", systemImage: "externaldrive")
            }
        }
        .formStyle(.grouped)
        .onAppear {
            Task { await service.fetchConfig() }
        }
    }
}

// MARK: - Agents Settings

struct AgentsSettingsView: View {
    @ObservedObject var service: ClawService
    @State private var showingAddAgent = false
    @State private var selectedAgent: ClawAgent?

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            if service.agents.isEmpty {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "person.2")
                        .font(.system(size: 32))
                        .foregroundStyle(.secondary)
                    Text("No agent profiles configured")
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity)
                Spacer()
            } else {
                List(selection: $selectedAgent) {
                    ForEach(service.agents) { agent in
                        AgentRow(agent: agent)
                            .tag(agent as ClawAgent?)
                            .contextMenu {
                                if agent.id != "default" {
                                    Divider()
                                    Button("Delete Agent", role: .destructive) {
                                        Task { await service.deleteAgent(agent.id) }
                                    }
                                }
                            }
                    }
                    .onDelete { indexSet in
                        for index in indexSet {
                            let agent = service.agents[index]
                            if agent.id != "default" {
                                Task { await service.deleteAgent(agent.id) }
                            }
                        }
                    }
                }
                .listStyle(.inset)
            }
        }
        .toolbar {
            ToolbarItemGroup {
                Button {
                    showingAddAgent = true
                } label: {
                    Label("Add Agent", systemImage: "plus")
                }
                .help("Add Agent Profile")
            }
        }
        .sheet(isPresented: $showingAddAgent) {
            AddAgentSheet(service: service)
        }
        .onAppear {
            Task { await service.fetchAgents() }
        }
    }
}

struct AgentRow: View {
    let agent: ClawAgent

    var body: some View {
        HStack(spacing: 10) {
            ZStack {
                RoundedRectangle(cornerRadius: 6)
                    .fill(agentGradient)
                    .frame(width: 32, height: 32)
                Image(systemName: agent.id == "default" ? "star.fill" : "person.fill")
                    .foregroundStyle(.white)
                    .font(.caption)
            }

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(agent.id)
                        .font(.body)
                        .fontWeight(.medium)
                    if agent.id == "default" {
                        Text("Default")
                            .font(.caption2)
                            .padding(.horizontal, 5)
                            .padding(.vertical, 1)
                            .background(Color.accentColor.opacity(0.12))
                            .cornerRadius(4)
                    }
                }

                HStack(spacing: 4) {
                    Text(agent.provider ?? "—")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                    if agent.provider != nil || agent.model != nil {
                        Text("·")
                            .foregroundStyle(.tertiary)
                    }
                    Text(agent.model ?? "")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }
            }

            Spacer()

            if let count = agent.toolCount {
                Text("\(count) tools")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 2)
                    .background(Color.secondary.opacity(0.08))
                    .cornerRadius(4)
            }
        }
        .padding(.vertical, 2)
    }

    private var agentGradient: LinearGradient {
        if agent.id == "default" {
            LinearGradient(colors: [.blue, .purple], startPoint: .topLeading, endPoint: .bottomTrailing)
        } else {
            LinearGradient(colors: [.teal, .mint], startPoint: .topLeading, endPoint: .bottomTrailing)
        }
    }
}

// MARK: - Add Agent Sheet

struct AddAgentSheet: View {
    @ObservedObject var service: ClawService
    @Environment(\.dismiss) private var dismiss
    @State private var id = ""
    @State private var provider = ""
    @State private var model = ""
    @State private var apiKey = ""
    @State private var baseURL = ""
    @State private var systemPrompt = ""

    var body: some View {
        VStack(spacing: 0) {
            // Header
            HStack {
                ZStack {
                    RoundedRectangle(cornerRadius: 8)
                        .fill(LinearGradient(colors: [.teal, .mint], startPoint: .topLeading, endPoint: .bottomTrailing))
                        .frame(width: 40, height: 40)
                    Image(systemName: "person.badge.plus")
                        .foregroundStyle(.white)
                        .font(.title3)
                }

                VStack(alignment: .leading, spacing: 2) {
                    Text("Add Agent Profile")
                        .font(.headline)
                    Text("Configure a new AI agent")
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }

                Spacer()
            }
            .padding()

            Divider()

            Form {
                TextField("Agent ID", text: $id)
                    .textFieldStyle(.roundedBorder)
                    .help("Unique identifier for this agent")

                TextField("Provider (e.g. openai)", text: $provider)
                    .textFieldStyle(.roundedBorder)

                TextField("Model (e.g. deepseek-v4-flash)", text: $model)
                    .textFieldStyle(.roundedBorder)

                SecureField("API Key (optional)", text: $apiKey)
                    .textFieldStyle(.roundedBorder)

                TextField("Base URL (optional)", text: $baseURL)
                    .textFieldStyle(.roundedBorder)

                TextField("System Prompt (optional)", text: $systemPrompt, axis: .vertical)
                    .textFieldStyle(.roundedBorder)
                    .lineLimit(3...6)
            }
            .formStyle(.grouped)
            .padding(.horizontal, 8)

            Divider()

            HStack {
                Button("Cancel") { dismiss() }
                    .keyboardShortcut(.escape)

                Spacer()

                Button {
                    Task {
                        await service.createAgent(
                            id: id,
                            provider: provider.isEmpty ? nil : provider,
                            model: model.isEmpty ? nil : model,
                            apiKey: apiKey.isEmpty ? nil : apiKey,
                            baseURL: baseURL.isEmpty ? nil : baseURL,
                            systemPrompt: systemPrompt.isEmpty ? nil : systemPrompt
                        )
                        await service.fetchAgents()
                        dismiss()
                    }
                } label: {
                    Text("Add Agent")
                        .frame(minWidth: 60)
                }
                .buttonStyle(.borderedProminent)
                .disabled(id.trimmingCharacters(in: .whitespaces).isEmpty)
            }
            .padding()
        }
    }
}

// MARK: - Backend Settings

struct BackendSettingsView: View {
    @ObservedObject var service: ClawService
    @State private var serverURL: String = ""
    @State private var authToken: String = ""

    var body: some View {
        Form {
            // Connection Card
            Section {
                HStack(spacing: 12) {
                    ZStack {
                        RoundedRectangle(cornerRadius: 8)
                            .fill(service.connectionState.isConnected
                                  ? Color.green.opacity(0.12)
                                  : Color.red.opacity(0.12))
                            .frame(width: 36, height: 36)
                        Image(systemName: service.connectionState.isConnected
                              ? "checkmark.circle.fill"
                              : "xmark.circle.fill")
                            .foregroundStyle(service.connectionState.isConnected ? .green : .red)
                            .font(.title3)
                    }

                    VStack(alignment: .leading, spacing: 2) {
                        Text("Status")
                            .font(.body)
                        Text(service.connectionState.isConnected
                             ? "All systems operational"
                             : service.connectionState.errorMessage ?? "Not connected")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }

                    Spacer()

                    if !service.connectionState.isConnected {
                        Button("Connect") {
                            service.restartBackend()
                        }
                        .controlSize(.small)
                    }
                }
                .padding(.vertical, 4)

                // Configurable Server URL
                HStack(spacing: 10) {
                    Image(systemName: "point.3.connected.trianglepath.dotted")
                        .foregroundStyle(.blue)
                        .font(.body)
                        .frame(width: 20)

                    VStack(alignment: .leading, spacing: 4) {
                        TextField("Server URL", text: $serverURL)
                            .textFieldStyle(.roundedBorder)
                            .font(.caption.monospaced())

                        HStack(spacing: 6) {
                            Button("Save & Reconnect") {
                                service.updateServerURL(serverURL)
                            }
                            .controlSize(.small)
                            .buttonStyle(.borderedProminent)
                            .disabled(serverURL.trimmingCharacters(in: .whitespaces).isEmpty)

                            Button("Reset") {
                                service.resetServerURL()
                            }
                            .controlSize(.small)
                        }
                    }
                }

                HStack(spacing: 10) {
                    Image(systemName: "key.fill")
                        .foregroundStyle(.orange)
                        .font(.body)
                        .frame(width: 20)

                    VStack(alignment: .leading, spacing: 4) {
                        SecureField("Auth Token (Bearer)", text: $authToken)
                            .textFieldStyle(.roundedBorder)
                            .font(.caption.monospaced())

                        HStack(spacing: 6) {
                            Button("Save & Reconnect") {
                                service.updateAuthToken(authToken)
                            }
                            .controlSize(.small)
                            .buttonStyle(.borderedProminent)
                            .disabled(authToken.trimmingCharacters(in: .whitespaces).isEmpty)

                            Button("Clear") {
                                authToken = ""
                                service.clearAuthToken()
                            }
                            .controlSize(.small)
                        }
                    }
                }
            } header: {
                Label("Connection", systemImage: "antenna.radiowaves.left.and.right")
            }

            // Sessions
            Section {
                SettingsRow(icon: "text.bubble", iconColor: .blue) {
                    LabeledContent("Total Sessions") {
                        Text("\(service.sessions.count)")
                            .foregroundStyle(.secondary)
                    }
                }

                HStack {
                    Image(systemName: "info.circle")
                        .foregroundStyle(.tertiary)
                        .font(.caption)
                    Text("Use Cmd+N to create a new session")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
                .padding(.leading, 32)
            } header: {
                Label("Sessions", systemImage: "list.bullet")
            }

            // Errors
            if let error = service.connectionState.errorMessage ?? service.errorMessage {
                Section {
                    HStack(spacing: 10) {
                        Image(systemName: "exclamationmark.triangle.fill")
                            .foregroundStyle(.orange)
                            .font(.title3)
                        Text(error)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.vertical, 2)
                } header: {
                    Label("Errors", systemImage: "exclamationmark.triangle")
                }
            }
        }
        .formStyle(.grouped)
        .onAppear {
            serverURL = service.serverURLDisplay
            authToken = UserDefaults.standard.string(forKey: "claw_auth_token") ?? ""
        }
        #if !os(macOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }
}

// MARK: - Reusable Settings Row

struct SettingsRow<Content: View>: View {
    let icon: String
    let iconColor: Color
    @ViewBuilder let content: Content

    var body: some View {
        HStack(spacing: 10) {
            Image(systemName: icon)
                .foregroundStyle(iconColor)
                .font(.body)
                .frame(width: 20)

            content
        }
    }
}
