import SwiftUI

struct PluginsPanel: View {
    @ObservedObject var service: ClawService
    @EnvironmentObject var appState: AppState

    var filteredPlugins: [PluginInfo] {
        guard !appState.searchText.isEmpty else { return service.plugins }
        return service.plugins.filter {
            $0.name.localizedCaseInsensitiveContains(appState.searchText) ||
            $0.description.localizedCaseInsensitiveContains(appState.searchText)
        }
    }

    var body: some View {
        Group {
            if service.plugins.isEmpty {
                emptyState
            } else {
                ScrollView {
                    LazyVStack(spacing: 10) {
                        ForEach(filteredPlugins) { plugin in
                            PluginCard(plugin: plugin)
                        }
                    }
                    .padding(20)
                }
            }
        }
        .onAppear {
            Task { await service.fetchPlugins() }
        }
    }

    private var emptyState: some View {
        VStack(spacing: 12) {
            Spacer()
            Image(systemName: "puzzlepiece")
                .font(.system(size: 36))
                .foregroundStyle(.secondary)
            Text("No plugins discovered")
                .font(.headline)
                .foregroundStyle(.secondary)
            Text("Add plugin manifests to ~/.i-rs-claw/plugins/")
                .font(.caption)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .frame(maxWidth: .infinity)
    }
}

struct PluginCard: View {
    let plugin: PluginInfo

    var body: some View {
        HStack(spacing: 12) {
            ZStack {
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .fill(plugin.enabled
                          ? LinearGradient(colors: [.blue, .cyan], startPoint: .topLeading, endPoint: .bottomTrailing)
                          : LinearGradient(colors: [Color.secondary.opacity(0.3), Color.secondary.opacity(0.15)], startPoint: .topLeading, endPoint: .bottomTrailing)
                    )
                    .frame(width: 36, height: 36)
                Image(systemName: plugin.enabled ? "puzzlepiece.fill" : "puzzlepiece")
                    .font(.system(size: 14, weight: .semibold))
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 3) {
                HStack(spacing: 6) {
                    Text(plugin.name)
                        .font(.callout)
                        .fontWeight(.medium)

                    Text("v\(plugin.version)")
                        .font(.caption2)
                        .foregroundStyle(.tertiary)
                        .padding(.horizontal, 4)
                        .padding(.vertical, 1)
                        .background(Color.secondary.opacity(0.08))
                        .clipShape(RoundedRectangle(cornerRadius: 3, style: .continuous))

                    if let author = plugin.author, !author.isEmpty {
                        Text(author)
                            .font(.caption2)
                            .foregroundStyle(.tertiary)
                    }
                }

                Text(plugin.description)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            Text(plugin.enabled ? "Active" : "Off")
                .font(.caption2)
                .fontWeight(.medium)
                .padding(.horizontal, 8)
                .padding(.vertical, 3)
                .background(
                    plugin.enabled ? Color.green.opacity(0.12) : Color.secondary.opacity(0.08)
                )
                .foregroundStyle(plugin.enabled ? .green : .secondary)
                .clipShape(Capsule())
        }
        .padding(12)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(.ultraThinMaterial)
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .strokeBorder(plugin.enabled ? Color.blue.opacity(0.15) : Color.secondary.opacity(0.1), lineWidth: 0.5)
        )
    }
}
