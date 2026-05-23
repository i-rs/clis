import SwiftUI

struct ToolsPanel: View {
    @ObservedObject var service: ClawService
    @EnvironmentObject var appState: AppState

    var filteredTools: [ToolInfo] {
        guard !appState.searchText.isEmpty else { return service.tools }
        return service.tools.filter {
            $0.name.localizedCaseInsensitiveContains(appState.searchText) ||
            $0.description.localizedCaseInsensitiveContains(appState.searchText)
        }
    }

    var groupedTools: [(String, [ToolInfo])] {
        let groups = Dictionary(grouping: filteredTools) { toolCategory($0.name) }
        return categoryOrder.compactMap { cat in
            groups[cat].map { (cat, $0) }
        }
    }

    var body: some View {
        Group {
            if service.tools.isEmpty {
                emptyState
            } else if filteredTools.isEmpty {
                noResults
            } else {
                ScrollView {
                    LazyVStack(alignment: .leading, spacing: 20) {
                        ForEach(groupedTools, id: \.0) { category, tools in
                            toolSection(category: category, tools: tools)
                        }
                    }
                    .padding(20)
                }
            }
        }
        .onAppear {
            Task { await service.fetchTools() }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Spacer()
            Image(systemName: "wrench.adjustable")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text("No tools available")
                .font(.headline)
                .foregroundStyle(.secondary)
            Text("Connect to the backend to see available tools")
                .font(.caption)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .frame(maxWidth: .infinity)
    }

    private var noResults: some View {
        VStack(spacing: 12) {
            Spacer()
            Image(systemName: "magnifyingglass")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text("No tools match your search")
                .foregroundStyle(.secondary)
            Spacer()
        }
        .frame(maxWidth: .infinity)
    }

    @ViewBuilder
    private func toolSection(category: String, tools: [ToolInfo]) -> some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 6) {
                Image(systemName: Self.categoryIcon(category))
                    .foregroundStyle(Self.categoryColor(category))
                    .font(.caption)
                Text(category)
                    .font(.caption)
                    .fontWeight(.semibold)
                    .foregroundStyle(.secondary)
                    .textCase(.uppercase)
                Text("\(tools.count)")
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
            }

            LazyVGrid(columns: [
                GridItem(.flexible(), spacing: 10),
                GridItem(.flexible(), spacing: 10)
            ], spacing: 10) {
                ForEach(tools) { tool in
                    ToolCard(tool: tool, category: category)
                }
            }
        }
    }

    private func toolCategory(_ name: String) -> String {
        if name.hasPrefix("i_rs") || name == "i-rs" { return "i-rs CLI" }
        if name.contains("search") || name.contains("web") { return "Search" }
        if name.contains("file") || name.contains("semantic") { return "Files" }
        if name.contains("memory") || name.contains("skill") { return "Memory" }
        if name.contains("chart") { return "Visualization" }
        if name.contains("delegate") { return "Agent" }
        if name.contains("vision") { return "Vision" }
        if name.hasPrefix("mcp_") { return "MCP" }
        return "Built-in"
    }

    private let categoryOrder = ["i-rs CLI", "Built-in", "Search", "Files", "Memory", "Vision", "Visualization", "Agent", "MCP"]

    static func categoryIcon(_ cat: String) -> String {
        switch cat {
        case "i-rs CLI": return "terminal"
        case "Search": return "magnifyingglass"
        case "Files": return "folder"
        case "Memory": return "brain"
        case "Vision": return "eye"
        case "Visualization": return "chart.bar"
        case "Agent": return "person.2"
        case "MCP": return "puzzlepiece"
        default: return "wrench.and.screwdriver"
        }
    }

    static func categoryColor(_ cat: String) -> Color {
        switch cat {
        case "i-rs CLI": return .orange
        case "Search": return .blue
        case "Files": return .teal
        case "Memory": return .purple
        case "Vision": return .indigo
        case "Visualization": return .green
        case "Agent": return .pink
        case "MCP": return .mint
        default: return .secondary
        }
    }
}

struct ToolCard: View {
    let tool: ToolInfo
    let category: String
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            HStack(spacing: 10) {
                ZStack {
                    RoundedRectangle(cornerRadius: 10, style: .continuous)
                        .fill(
                            LinearGradient(
                                colors: [
                                    ToolsPanel.categoryColor(category).opacity(colorScheme == .dark ? 0.3 : 0.2),
                                    ToolsPanel.categoryColor(category).opacity(colorScheme == .dark ? 0.2 : 0.1)
                                ],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                        )
                        .frame(width: 36, height: 36)
                    Image(systemName: ToolsPanel.categoryIcon(category))
                        .font(.system(size: 14, weight: .semibold))
                        .foregroundStyle(ToolsPanel.categoryColor(category))
                }
                .shadow(color: ToolsPanel.categoryColor(category).opacity(colorScheme == .dark ? 0.3 : 0.2), radius: 4, x: 0, y: 2)

                Text(tool.name)
                    .font(.callout)
                    .fontWeight(.medium)
                    .lineLimit(1)
            }

            Text(tool.description)
                .font(.caption)
                .foregroundStyle(.primary.opacity(0.7))
                .lineLimit(3)
                .fixedSize(horizontal: false, vertical: true)
        }
        .padding(14)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            LinearGradient(
                colors: colorScheme == .dark
                ? [Color(white: 0.18), Color(white: 0.15)]
                : [Color(white: 0.99), Color(white: 0.97)],
                startPoint: .topLeading,
                endPoint: .bottomTrailing
            )
        )
        .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        .shadow(color: colorScheme == .dark ? .black.opacity(0.3) : .black.opacity(0.05), radius: 8, x: 0, y: 3)
        .overlay(
            RoundedRectangle(cornerRadius: 14, style: .continuous)
                .strokeBorder(
                    LinearGradient(
                        colors: [
                            ToolsPanel.categoryColor(category).opacity(colorScheme == .dark ? 0.25 : 0.15),
                            ToolsPanel.categoryColor(category).opacity(colorScheme == .dark ? 0.1 : 0.05)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ),
                    lineWidth: 1
                )
        )
    }
}
