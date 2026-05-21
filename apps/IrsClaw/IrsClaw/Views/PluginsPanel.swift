import SwiftUI

struct PluginsPanel: View {
    @ObservedObject var service: ClawService
    @State private var searchText = ""

    var filteredPlugins: [PluginInfo] {
        guard !searchText.isEmpty else { return service.plugins }
        return service.plugins.filter {
            $0.name.localizedCaseInsensitiveContains(searchText) ||
            $0.description.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            if service.plugins.isEmpty {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "puzzlepiece")
                        .font(.system(size: 32))
                        .foregroundStyle(.secondary)
                    Text("No plugins discovered")
                        .foregroundStyle(.secondary)
                    Text("Add plugin manifests to ~/.i-rs-claw/plugins/")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity)
                Spacer()
            } else {
                List(filteredPlugins) { plugin in
                    PluginRow(plugin: plugin)
                }
                .listStyle(.inset)
            }
        }
        .searchable(text: $searchText, prompt: "Search plugins")
        .onAppear {
            Task { await service.fetchPlugins() }
        }
    }
}

struct PluginRow: View {
    let plugin: PluginInfo

    var body: some View {
        HStack(spacing: 10) {
            ZStack {
                RoundedRectangle(cornerRadius: 6)
                    .fill(plugin.enabled ? Color.blue : Color.secondary.opacity(0.2))
                    .frame(width: 32, height: 32)
                Image(systemName: plugin.enabled ? "puzzlepiece.fill" : "puzzlepiece")
                    .foregroundStyle(.white)
                    .font(.caption)
            }

            VStack(alignment: .leading, spacing: 2) {
                HStack(spacing: 6) {
                    Text(plugin.name)
                        .font(.body)
                        .fontWeight(.medium)

                    Text("v\(plugin.version)")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }

                Text(plugin.description)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(1)
            }

            Spacer()

            Text(plugin.enabled ? "Active" : "Inactive")
                .font(.caption)
                .padding(.horizontal, 8)
                .padding(.vertical, 2)
                .background(plugin.enabled ? Color.blue.opacity(0.12) : Color.secondary.opacity(0.08))
                .foregroundStyle(plugin.enabled ? .blue : .secondary)
                .cornerRadius(4)
        }
        .padding(.vertical, 2)
    }
}
