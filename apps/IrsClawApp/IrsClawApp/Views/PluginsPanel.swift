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
            Text("Add plugin manifests to ~/.i-rs/claw/plugins/")
                .font(.caption)
                .foregroundStyle(.tertiary)
            Spacer()
        }
        .frame(maxWidth: .infinity)
    }
}

struct PluginCard: View {
    let plugin: PluginInfo
    @Environment(\.colorScheme) private var colorScheme

    var body: some View {
        HStack(spacing: 14) {
            ZStack {
                RoundedRectangle(cornerRadius: 10, style: .continuous)
                    .fill(
                        plugin.enabled
                        ? LinearGradient(
                            colors: [.blue, .cyan],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                        : LinearGradient(
                            colors: [
                                Color.secondary.opacity(colorScheme == .dark ? 0.5 : 0.4),
                                Color.secondary.opacity(colorScheme == .dark ? 0.35 : 0.25)
                            ],
                            startPoint: .topLeading,
                            endPoint: .bottomTrailing
                        )
                    )
                    .frame(width: 44, height: 44)
                    .shadow(color: plugin.enabled ? .blue.opacity(colorScheme == .dark ? 0.4 : 0.3) : .clear, radius: 4, x: 0, y: 2)
                Image(systemName: plugin.enabled ? "puzzlepiece.fill" : "puzzlepiece")
                    .font(.system(size: 16, weight: .semibold))
                    .foregroundStyle(.white)
            }

            VStack(alignment: .leading, spacing: 4) {
                HStack(spacing: 8) {
                    Text(plugin.name)
                        .font(.callout)
                        .fontWeight(.medium)

                    Text("v\(plugin.version)")
                        .font(.caption2)
                        .foregroundStyle(.secondary)
                        .padding(.horizontal, 6)
                        .padding(.vertical, 2)
                        .background(Color.secondary.opacity(colorScheme == .dark ? 0.15 : 0.1))
                        .clipShape(RoundedRectangle(cornerRadius: 4, style: .continuous))

                    if let author = plugin.author, !author.isEmpty {
                        Text(author)
                            .font(.caption)
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
                .font(.caption)
                .fontWeight(.medium)
                .padding(.horizontal, 10)
                .padding(.vertical, 5)
                .background(
                    plugin.enabled
                    ? LinearGradient(
                        colors: [
                            .green.opacity(colorScheme == .dark ? 0.25 : 0.15),
                            .green.opacity(colorScheme == .dark ? 0.15 : 0.08)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                    : LinearGradient(
                        colors: [
                            Color.secondary.opacity(colorScheme == .dark ? 0.15 : 0.1),
                            Color.secondary.opacity(colorScheme == .dark ? 0.1 : 0.05)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                )
                .foregroundStyle(plugin.enabled ? .green : .secondary)
                .clipShape(Capsule())
        }
        .padding(14)
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
                    plugin.enabled
                    ? LinearGradient(
                        colors: [
                            .blue.opacity(colorScheme == .dark ? 0.25 : 0.15),
                            .cyan.opacity(colorScheme == .dark ? 0.15 : 0.08)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    )
                    : LinearGradient(
                        colors: [
                            Color.secondary.opacity(colorScheme == .dark ? 0.15 : 0.1),
                            Color.secondary.opacity(colorScheme == .dark ? 0.1 : 0.05)
                        ],
                        startPoint: .topLeading,
                        endPoint: .bottomTrailing
                    ),
                    lineWidth: 1
                )
        )
    }
}
