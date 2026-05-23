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

    var body: some View {
        #if os(macOS)
        NavigationSplitView {
            sidebarContent
        } detail: {
            detailView
        }
        .frame(width: 640, height: 480)
        #else
        iPhoneSettingsView
        #endif
    }

    // MARK: - iOS Settings

    #if os(iOS)
    private var iPhoneSettingsView: some View {
        NavigationStack {
            List {
                Section {
                    serverURLRow
                } header: {
                    Text("Server")
                } footer: {
                    Text("Use your Mac's local IP (e.g., 192.168.1.100) for real device testing")
                        .font(.caption)
                }

                Section {
                    connectionStatusRow
                }

                Section("AI Provider") {
                    NavigationLink {
                        LLMProviderSettingsView(service: service)
                    } label: {
                        providerLabel
                    }
                }

                Section("Agents") {
                    NavigationLink {
                        AgentsSettingsView(service: service)
                    } label: {
                        Label("Manage Agents", systemImage: "person.2")
                    }
                }

                Section("Backend") {
                    NavigationLink {
                        BackendSettingsView(service: service)
                    } label: {
                        Label("Server & Authentication", systemImage: "server.rack")
                    }
                }

                Section("About") {
                    HStack {
                        Text("Version")
                        Spacer()
                        Text("1.0.0")
                            .foregroundStyle(.secondary)
                    }
                }
            }
            .navigationTitle("Settings")
            .navigationBarTitleDisplayMode(.large)
            .toolbar {
                ToolbarItem(placement: .confirmationAction) {
                    Button("Done") { dismiss() }
                }
            }
        }
    }

    private var connectionStatusRow: some View {
        HStack(spacing: 12) {
            Circle()
                .fill(service.connectionState.isConnected ? Color.green : Color.red)
                .frame(width: 10, height: 10)

            VStack(alignment: .leading, spacing: 2) {
                Text(service.connectionState.isConnected ? "Connected" : "Disconnected")
                    .font(.body)
                Text(service.connectionState.isConnected ? "i-rs-claw backend" : (service.connectionState.errorMessage ?? "Not connected"))
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }

            Spacer()

            if !service.connectionState.isConnected {
                Button("Connect") {
                    service.restartBackend()
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
        }
        .padding(.vertical, 4)
    }

    private var serverURLRow: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack(spacing: 12) {
                Image(systemName: "server.rack")
                    .foregroundStyle(.blue)
                    .frame(width: 24)

                VStack(alignment: .leading, spacing: 2) {
                    Text("Server Address")
                        .font(.body)
                    Text(service.serverURLDisplay)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                Button("Change") {
                    showingServerURLSheet = true
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
        }
        .padding(.vertical, 4)
        .sheet(isPresented: $showingServerURLSheet) {
            ServerURLSheet(service: service)
        }
    }

    @State private var showingServerURLSheet = false

    private var providerLabel: some View {
        HStack(spacing: 12) {
            if let config = service.config {
                Image(systemName: providerIcon(config.provider ?? ""))
                    .foregroundStyle(providerColor(config.provider ?? ""))
                    .frame(width: 24)

                VStack(alignment: .leading, spacing: 2) {
                    Text(providerName(config.provider ?? ""))
                        .font(.body)
                    if let model = config.model {
                        Text(model)
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                }
            } else {
                Label("Configure Provider", systemImage: "gear")
                    .foregroundStyle(.secondary)
            }
        }
    }

    private func providerIcon(_ id: String) -> String {
        switch id {
        case "openai": return "brain"
        case "anthropic": return "a.circle"
        case "deepseek": return "magnifyingglass"
        case "minimax": return "bolt"
        case "zhipu": return "z.circle"
        case "kimi": return "k.circle"
        case "aliyun": return "a.circle"
        case "ollama": return "llama"
        default: return "cpu"
        }
    }

    private func providerColor(_ id: String) -> Color {
        switch id {
        case "openai": return .green
        case "anthropic": return .orange
        case "deepseek": return .blue
        case "minimax": return .purple
        case "zhipu": return .cyan
        case "kimi": return .pink
        case "aliyun": return .indigo
        case "ollama": return .teal
        default: return .secondary
        }
    }

    private func providerName(_ id: String) -> String {
        switch id {
        case "openai": return "OpenAI"
        case "anthropic": return "Anthropic"
        case "deepseek": return "DeepSeek"
        case "minimax": return "MiniMax"
        case "zhipu": return "Zhipu GLM"
        case "kimi": return "Kimi"
        case "aliyun": return "Aliyun"
        case "ollama": return "Ollama"
        default: return id.capitalized
        }
    }
    #endif

    // MARK: - macOS Settings

    #if os(macOS)
    @ViewBuilder
    private var sidebarContent: some View {
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
                .padding(.vertical, 2)
                .tag(section)
            }
        }
        .listStyle(.sidebar)
        .navigationSplitViewColumnWidth(200)
    }

    @State private var selectedSection: SettingsSection? = .general

    @ViewBuilder
    private var detailView: some View {
        switch selectedSection ?? .general {
        case .general: GeneralSettingsView(service: service)
        case .agents: AgentsSettingsView(service: service)
        case .backend: BackendSettingsView(service: service)
        }
    }
    #endif
}

// MARK: - Server URL Sheet (iPhone)

struct ServerURLSheet: View {
    @ObservedObject var service: ClawService
    @Environment(\.dismiss) private var dismiss
    @State private var serverURL: String = ""

    var body: some View {
        NavigationStack {
            List {
                Section {
                    TextField("http://192.168.1.100:3000", text: $serverURL)
                } header: {
                    Text("Server Address")
                } footer: {
                    Text("Find your Mac's IP: System Settings > Wi-Fi > [Network] > IP Address")
                        .font(.caption)
                }

                Section {
                    Button("Use 127.0.0.1 (Simulator/Mac)") {
                        serverURL = "http://127.0.0.1:3000"
                    }
                    .foregroundStyle(.secondary)
                }
            }
            .navigationTitle("Server URL")
            #if os(iOS)
            .navigationBarTitleDisplayMode(.inline)
            #endif
            .toolbar {
                ToolbarItem(placement: .cancellationAction) {
                    Button("Cancel") { dismiss() }
                }
                ToolbarItem(placement: .confirmationAction) {
                    Button("Save") {
                        saveServerURL()
                    }
                    .disabled(serverURL.trimmingCharacters(in: .whitespaces).isEmpty)
                }
            }
        }
        .onAppear {
            serverURL = service.serverURLDisplay
        }
    }

    private func saveServerURL() {
        let url = serverURL.trimmingCharacters(in: .whitespaces)
        guard !url.isEmpty else { return }
        service.updateServerURL(url)
        dismiss()
    }
}

// MARK: - LLM Provider Settings (iPhone)

struct LLMProviderSettingsView: View {
    @ObservedObject var service: ClawService
    @AppStorage("llm_provider") private var llmProvider: String = "openai"
    @AppStorage("llm_base_url") private var llmBaseURL: String = ""
    @AppStorage("llm_api_key") private var llmAPIKey: String = ""

    var body: some View {
        List {
            Section("Provider") {
                Picker("AI Provider", selection: $llmProvider) {
                    ForEach(ProviderOption.allCases) { opt in
                        Label(opt.name, systemImage: opt.icon).tag(opt.id)
                    }
                }
                .pickerStyle(.inline)
                .labelsHidden()
            }

            Section("Configuration") {
                SecureField("API Key", text: $llmAPIKey)
                    .textContentType(.password)

                TextField("Base URL (optional)", text: $llmBaseURL)
            }

            Section {
                Button("Save & Reconnect") {
                    saveAndReconnect()
                }
                .frame(maxWidth: .infinity)
                .foregroundStyle(.white)
                .listRowBackground(Color.accentColor)
            }
        }
        .navigationTitle("AI Provider")
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    private func saveAndReconnect() {
        UserDefaults.standard.set(llmProvider, forKey: "llm_provider")
        UserDefaults.standard.set(llmAPIKey, forKey: "llm_api_key")
        UserDefaults.standard.set(llmBaseURL, forKey: "llm_base_url")

        Task {
            await service.updateLLMConfig(
                provider: llmProvider,
                apiKey: llmAPIKey,
                baseURL: llmBaseURL
            )
            service.restartBackend()
        }
    }
}

// MARK: - General Settings

struct GeneralSettingsView: View {
    @ObservedObject var service: ClawService
    @AppStorage("app_appearance") private var appearance: String = "system"
    @AppStorage("llm_provider") private var llmProvider: String = "openai"
    @AppStorage("llm_base_url") private var llmBaseURL: String = ""
    @AppStorage("llm_api_key") private var llmAPIKey: String = ""
    @State private var editingProvider = false

    var body: some View {
        Form {
            Section {
                connectionStatusCard
            } header: {
                Text("Connection")
            }

            Section("Appearance") {
                ThemePicker(selection: $appearance)
                    .onChange(of: appearance) { _, newValue in
                        applyAppearance(newValue)
                    }
            }

            Section {
                providerRow
                if editingProvider {
                    providerEditRow
                }
            } header: {
                Label("LLM Provider", systemImage: "brain")
            }

            if let config = service.config {
                Section("LLM Configuration") {
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
                }
            }
        }
        .formStyle(.grouped)
        .onAppear {
            applyAppearance(appearance)
            Task { await service.fetchConfig() }
        }
    }

    @ViewBuilder
    private var providerRow: some View {
        HStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(
                        LinearGradient(
                            colors: [providerColor(llmProvider).opacity(0.2), providerColor(llmProvider).opacity(0.1)],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 36, height: 36)
                    .shadow(color: providerColor(llmProvider).opacity(0.2), radius: 3, x: 0, y: 2)
                Image(systemName: providerIcon(llmProvider))
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundStyle(providerColor(llmProvider))
            }

            VStack(alignment: .leading, spacing: 3) {
                Text(providerName(llmProvider))
                    .font(.callout)
                    .fontWeight(.medium)

                HStack(spacing: 6) {
                    if !llmAPIKey.isEmpty {
                        HStack(spacing: 3) {
                            Image(systemName: "lock.fill")
                                .font(.system(size: 9))
                            Text(maskedAPIKey)
                                .font(.caption)
                        }
                        .foregroundStyle(.green)
                    }
                    if !llmBaseURL.isEmpty {
                        Text(llmBaseURL)
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .lineLimit(1)
                    }
                }
            }

            Spacer()

            Button {
                withAnimation(.easeInOut(duration: 0.2)) {
                    editingProvider.toggle()
                }
            } label: {
                ZStack {
                    Circle()
                        .fill(Color.secondary.opacity(0.1))
                        .frame(width: 28, height: 28)
                    Image(systemName: editingProvider ? "chevron.up" : "pencil")
                        .font(.system(size: 12, weight: .medium))
                        .foregroundStyle(.secondary)
                }
            }
            .buttonStyle(.plain)
        }
        .padding(10)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(providerColor(llmProvider).opacity(0.05))
        )
    }

    @ViewBuilder
    private var providerEditRow: some View {
        VStack(spacing: 10) {
            Picker("Provider", selection: $llmProvider) {
                ForEach(ProviderOption.allCases, id: \.id) { opt in
                    Label(opt.name, systemImage: opt.icon).tag(opt.id)
                }
            }
            .pickerStyle(.menu)

            SecureField("API Key", text: $llmAPIKey)
                .textFieldStyle(.roundedBorder)
                .font(.callout.monospaced())
                .textContentType(.password)

            TextField("Base URL (optional)", text: $llmBaseURL)
                .textFieldStyle(.roundedBorder)
                .font(.callout.monospaced())
                .textContentType(.URL)

            HStack {
                Button("Save & Reconnect") {
                    UserDefaults.standard.set(llmProvider, forKey: "llm_provider")
                    UserDefaults.standard.set(llmAPIKey, forKey: "llm_api_key")
                    UserDefaults.standard.set(llmBaseURL, forKey: "llm_base_url")
                    Task {
                        await service.updateLLMConfig(
                            provider: llmProvider,
                            apiKey: llmAPIKey,
                            baseURL: llmBaseURL
                        )
                        service.restartBackend()
                    }
                    editingProvider = false
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)

                Button("Cancel") {
                    editingProvider = false
                }
                .controlSize(.small)

                Spacer()
            }
        }
        .padding(.leading, 38)
        .transition(.opacity.combined(with: .move(edge: .top)))
    }

    private var maskedAPIKey: String {
        guard !llmAPIKey.isEmpty else { return "" }
        if llmAPIKey.count <= 8 {
            return String(repeating: "•", count: llmAPIKey.count)
        }
        return String(llmAPIKey.prefix(4)) + String(repeating: "•", count: min(llmAPIKey.count - 8, 8)) + String(llmAPIKey.suffix(4))
    }

    private func providerIcon(_ id: String) -> String {
        switch id {
        case "openai": return "brain"
        case "anthropic": return "a.circle"
        case "deepseek": return "magnifyingglass"
        case "minimax": return "bolt"
        case "zhipu": return "z.circle"
        case "kimi": return "k.circle"
        case "aliyun": return "a.circle"
        case "ollama": return "llama"
        default: return "cpu"
        }
    }

    private func providerColor(_ id: String) -> Color {
        switch id {
        case "openai": return .green
        case "anthropic": return .orange
        case "deepseek": return .blue
        case "minimax": return .purple
        case "zhipu": return .cyan
        case "kimi": return .pink
        case "aliyun": return .indigo
        case "ollama": return .teal
        default: return .secondary
        }
    }

    private func providerName(_ id: String) -> String {
        ProviderOption.allCases.first(where: { $0.id == id })?.name ?? id
    }

    @ViewBuilder
    private var connectionStatusCard: some View {
        HStack(spacing: 14) {
            ZStack {
                Circle()
                    .fill(
                        service.connectionState.isConnected
                        ? LinearGradient(colors: [.green.opacity(0.2), .green.opacity(0.1)], startPoint: .topLeading, endPoint: .bottomTrailing)
                        : LinearGradient(colors: [.red.opacity(0.2), .red.opacity(0.1)], startPoint: .topLeading, endPoint: .bottomTrailing)
                    )
                    .frame(width: 44, height: 44)
                Image(systemName: service.connectionState.isConnected ? "checkmark.circle.fill" : "exclamationmark.circle.fill")
                    .foregroundStyle(service.connectionState.isConnected ? .green : .red)
                    .font(.title2)
            }

            VStack(alignment: .leading, spacing: 3) {
                Text(service.connectionState.isConnected ? "Connected" : "Disconnected")
                    .font(.body)
                    .fontWeight(.medium)
                Text(service.connectionState.isConnected
                     ? "Connected to i-rs-claw backend"
                     : service.connectionState.errorMessage ?? "Not connected")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            if !service.connectionState.isConnected {
                Button("Reconnect") {
                    service.restartBackend()
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
            }
        }
        .padding(12)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(service.connectionState.isConnected ? Color.green.opacity(0.05) : Color.red.opacity(0.05))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .strokeBorder(service.connectionState.isConnected ? Color.green.opacity(0.15) : Color.red.opacity(0.15), lineWidth: 1)
        )
    }

    private func applyAppearance(_ mode: String) {
        #if os(macOS)
        switch mode {
        case "light":
            NSApp.appearance = NSAppearance(named: .aqua)
        case "dark":
            NSApp.appearance = NSAppearance(named: .darkAqua)
        default:
            NSApp.appearance = nil
        }
        #endif
    }
}

enum ProviderOption: String, CaseIterable, Identifiable {
    case openai, anthropic, deepseek, minimax, zhipu, kimi, aliyun, ollama

    var id: String { rawValue }

    var name: String {
        switch self {
        case .openai: return "OpenAI"
        case .anthropic: return "Anthropic"
        case .deepseek: return "DeepSeek"
        case .minimax: return "MiniMax"
        case .zhipu: return "Zhipu GLM"
        case .kimi: return "Kimi (Moonshot)"
        case .aliyun: return "Aliyun (Qwen)"
        case .ollama: return "Ollama (Local)"
        }
    }

    var icon: String {
        switch self {
        case .openai: return "brain"
        case .anthropic: return "a.circle"
        case .deepseek: return "magnifyingglass"
        case .minimax: return "bolt"
        case .zhipu: return "z.circle"
        case .kimi: return "k.circle"
        case .aliyun: return "a.circle"
        case .ollama: return "llama"
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
            HStack {
                Image(systemName: "person.badge.plus")
                    .foregroundStyle(Color.blue)
                    .font(.title3)

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
                Section("Identity") {
                    TextField("Agent ID", text: $id)
                        .help("Unique identifier for this agent")
                }

                Section("Model") {
                    TextField("Provider (e.g. openai)", text: $provider)
                    TextField("Model (e.g. gpt-4o-mini)", text: $model)
                }

                Section("Authentication") {
                    SecureField("API Key (optional)", text: $apiKey)
                    TextField("Base URL (optional)", text: $baseURL)
                }

                Section("Behavior") {
                    TextEditor(text: $systemPrompt)
                        .frame(minHeight: 60)
                        .font(.caption.monospaced())
                }
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
                .keyboardShortcut(.return, modifiers: .command)
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
    @State private var editingServer = false
    @State private var editingToken = false

    var body: some View {
        Form {
            Section {
                connectionStatusCard
            }

            Section {
                serverRow
                if editingServer {
                    serverEditRow
                }
            } header: {
                Label("Server", systemImage: "server.rack")
            }

            Section {
                tokenRow
                if editingToken {
                    tokenEditRow
                }
            } header: {
                Label("Authentication", systemImage: "key.fill")
            }

            Section {
                SettingsRow(icon: "text.bubble", iconColor: .blue) {
                    LabeledContent("Total Sessions") {
                        Text("\(service.sessions.count)")
                            .foregroundStyle(.secondary)
                    }
                }
            } header: {
                Label("Data", systemImage: "externaldrive")
            } footer: {
                Text("⌘N to create a new session")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }

            if let error = service.connectionState.errorMessage ?? service.errorMessage {
                Section {
                    Label(error, systemImage: "exclamationmark.triangle.fill")
                        .font(.caption)
                        .foregroundStyle(.orange)
                }
            }
        }
        .formStyle(.grouped)
        .onAppear {
            serverURL = service.serverURLDisplay
            authToken = UserDefaults.standard.string(forKey: "claw_auth_token") ?? ""
        }
        #if os(iOS)
        .navigationBarTitleDisplayMode(.inline)
        #endif
    }

    @ViewBuilder
    private var connectionStatusCard: some View {
        HStack(spacing: 12) {
            Circle()
                .fill(service.connectionState.isConnected ? Color.green : Color.red)
                .frame(width: 10, height: 10)
                .overlay(
                    Circle()
                        .fill(service.connectionState.isConnected ? Color.green.opacity(0.3) : Color.red.opacity(0.3))
                        .frame(width: 20, height: 20)
                )

            VStack(alignment: .leading, spacing: 2) {
                Text(service.connectionState.isConnected ? "Connected" : "Disconnected")
                    .font(.body)
                    .fontWeight(.medium)
                Text(service.connectionState.isConnected
                     ? "All systems operational"
                     : service.connectionState.errorMessage ?? "Not connected")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            if !service.connectionState.isConnected {
                Button("Connect") {
                    service.restartBackend()
                }
                .buttonStyle(.bordered)
                .controlSize(.small)
            }
        }
        .padding(.vertical, 4)
    }

    @ViewBuilder
    private var serverRow: some View {
        HStack(spacing: 10) {
            Image(systemName: "link")
                .foregroundStyle(.secondary)
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 2) {
                Text("URL")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                Text(service.serverURLDisplay)
                    .font(.callout.monospaced())
                    .lineLimit(1)
            }

            Spacer()

            Button {
                withAnimation(.easeInOut(duration: 0.15)) {
                    editingServer.toggle()
                    editingToken = false
                }
            } label: {
                Image(systemName: editingServer ? "chevron.up" : "pencil")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
    }

    @ViewBuilder
    private var serverEditRow: some View {
        VStack(spacing: 10) {
            TextField("http://127.0.0.1:3000", text: $serverURL)
                .textFieldStyle(.roundedBorder)
                .font(.callout.monospaced())
                .textContentType(.URL)

            HStack {
                Button("Save & Reconnect") {
                    service.updateServerURL(serverURL)
                    editingServer = false
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(serverURL.trimmingCharacters(in: .whitespaces).isEmpty)

                Button("Reset") {
                    serverURL = "http://127.0.0.1:3000"
                    service.resetServerURL()
                    editingServer = false
                }
                .controlSize(.small)

                Spacer()
            }
        }
        .padding(.leading, 26)
        .transition(.opacity.combined(with: .move(edge: .top)))
    }

    @ViewBuilder
    private var tokenRow: some View {
        HStack(spacing: 10) {
            Image(systemName: authToken.isEmpty ? "lock.open" : "lock.fill")
                .foregroundStyle(authToken.isEmpty ? .orange : .green)
                .frame(width: 16)

            VStack(alignment: .leading, spacing: 2) {
                Text("Dashboard Token")
                    .font(.caption)
                    .foregroundStyle(.secondary)
                if authToken.isEmpty {
                    Text("Not configured")
                        .font(.callout)
                        .foregroundStyle(.tertiary)
                } else {
                    Text(maskedToken)
                        .font(.callout.monospaced())
                }
            }

            Spacer()

            Button {
                withAnimation(.easeInOut(duration: 0.15)) {
                    editingToken.toggle()
                    editingServer = false
                }
            } label: {
                Image(systemName: editingToken ? "chevron.up" : "pencil")
                    .font(.caption)
                    .foregroundStyle(.secondary)
            }
            .buttonStyle(.plain)
        }
    }

    private var maskedToken: String {
        guard !authToken.isEmpty else { return "" }
        if authToken.count <= 8 {
            return String(repeating: "•", count: authToken.count)
        }
        return String(authToken.prefix(4)) + String(repeating: "•", count: min(authToken.count - 8, 8)) + String(authToken.suffix(4))
    }

    @ViewBuilder
    private var tokenEditRow: some View {
        VStack(spacing: 10) {
            SecureField("Enter dashboard token", text: $authToken)
                .textFieldStyle(.roundedBorder)
                .font(.callout.monospaced())

            HStack(spacing: 8) {
                Button("Save & Reconnect") {
                    service.updateAuthToken(authToken)
                    editingToken = false
                }
                .buttonStyle(.borderedProminent)
                .controlSize(.small)
                .disabled(authToken.trimmingCharacters(in: .whitespaces).isEmpty)

                Button("Clear") {
                    authToken = ""
                    service.clearAuthToken()
                    editingToken = false
                }
                .controlSize(.small)

                Spacer()
            }
        }
        .padding(.leading, 26)
        .transition(.opacity.combined(with: .move(edge: .top)))
    }
}

// MARK: - Theme Picker

struct ThemePicker: View {
    @Binding var selection: String

    private let options: [(id: String, icon: String, label: String)] = [
        ("light", "sun.max.fill", "Light"),
        ("system", "desktopcomputer", "Auto"),
        ("dark", "moon.fill", "Dark"),
    ]

    var body: some View {
        HStack(spacing: 6) {
            ForEach(options, id: \.id) { opt in
                Button {
                    selection = opt.id
                } label: {
                    VStack(spacing: 4) {
                        ZStack {
                            Circle()
                                .fill(selection == opt.id ? Color.accentColor : Color.secondary.opacity(0.1))
                                .frame(width: 32, height: 32)
                            Image(systemName: opt.icon)
                                .font(.system(size: 14, weight: .medium))
                                .foregroundStyle(selection == opt.id ? .white : .secondary)
                        }
                        Text(opt.label)
                            .font(.caption2)
                            .foregroundStyle(selection == opt.id ? .primary : .secondary)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 6)
                }
                .buttonStyle(.plain)
            }
        }
        .padding(4)
        .background(Color.secondary.opacity(0.06))
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
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
