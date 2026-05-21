import SwiftUI

struct ToolsPanel: View {
    @ObservedObject var service: ClawService
    @State private var searchText = ""

    var filteredTools: [ToolInfo] {
        guard !searchText.isEmpty else { return service.tools }
        return service.tools.filter {
            $0.name.localizedCaseInsensitiveContains(searchText) ||
            $0.description.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            if service.tools.isEmpty {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "wrench.adjustable")
                        .font(.system(size: 32))
                        .foregroundStyle(.secondary)
                    Text("No tools available")
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity)
                Spacer()
            } else {
                List(filteredTools) { tool in
                    ToolRow(tool: tool)
                }
                .listStyle(.inset)
            }
        }
        .searchable(text: $searchText, prompt: "Search tools")
        .onAppear {
            Task { await service.fetchTools() }
        }
    }
}

struct ToolRow: View {
    let tool: ToolInfo

    var body: some View {
        VStack(alignment: .leading, spacing: 6) {
            HStack(spacing: 8) {
                Image(systemName: "wrench.and.screwdriver")
                    .foregroundStyle(.orange)
                    .font(.title3)
                    .frame(width: 24)

                Text(tool.name)
                    .font(.body)
                    .fontWeight(.medium)

                Spacer()
            }

            Text(tool.description)
                .font(.caption)
                .foregroundStyle(.secondary)
                .lineLimit(2)
        }
        .padding(.vertical, 4)
    }
}
