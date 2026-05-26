import SwiftUI

struct UsagePanel: View {
    @ObservedObject var service: ClawService

    var body: some View {
        let total = service.totalTokenUsage
        List {
            Section {
                VStack(spacing: 16) {
                    totalRow(icon: "number", label: "Total Tokens", value: "\(total.totalTokens)")
                        .font(.title2.weight(.semibold))

                    HStack(spacing: 24) {
                        statItem(label: "Prompt", value: "\(total.promptTokens ?? 0)", color: .blue)
                        statItem(label: "Completion", value: "\(total.completionTokens ?? 0)", color: .green)
                        statItem(label: "Sessions", value: "\(service.sessionTokenUsage.count)", color: .orange)
                    }
                }
                .padding(.vertical, 8)
                .frame(maxWidth: .infinity)
            }

            if !usageSessions.isEmpty {
                Section("Per Session") {
                    ForEach(usageSessions, id: \.id) { session in
                        HStack {
                            VStack(alignment: .leading, spacing: 2) {
                                Text(session.session?.title ?? session.id)
                                    .font(.callout)
                                    .fontWeight(.medium)
                                    .lineLimit(1)
                                if let s = session.session {
                                    Text(s.shortDate)
                                        .font(.caption)
                                        .foregroundStyle(.tertiary)
                                }
                            }

                            Spacer()

                            Text("\(session.usage.totalTokens) tokens")
                                .font(.callout.monospacedDigit())
                                .foregroundStyle(.secondary)
                        }
                        .padding(.vertical, 2)
                    }
                }
            }

            if service.sessionTokenUsage.isEmpty {
                Section {
                    VStack(spacing: 8) {
                        Image(systemName: "chart.bar")
                            .font(.system(size: 32))
                            .foregroundStyle(.tertiary)
                        Text("No token usage data yet")
                            .foregroundStyle(.secondary)
                        Text("Token usage is tracked per session after each chat response")
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .multilineTextAlignment(.center)
                    }
                    .frame(maxWidth: .infinity)
                    .padding(.vertical, 24)
                }
            }
        }
        #if !os(macOS)
        .listStyle(.insetGrouped)
        #else
        .listStyle(.inset)
        #endif
    }

    private var usageSessions: [(id: String, session: ClawSession?, usage: TokenUsage)] {
        service.sessionTokenUsage
            .compactMap { id, usage in
                let session = service.sessions.first { $0.id == id }
                return (id, session, usage)
            }
            .sorted { a, b in
                let da = a.session?.dateValue ?? .distantPast
                let db = b.session?.dateValue ?? .distantPast
                return da > db
            }
    }

    private func totalRow(icon: String, label: String, value: String) -> some View {
        HStack(spacing: 8) {
            Image(systemName: icon)
                .foregroundStyle(.blue)
            Text(label)
            Text(value)
                .monospacedDigit()
        }
    }

    private func statItem(label: String, value: String, color: Color) -> some View {
        VStack(spacing: 4) {
            Text(value)
                .font(.title3.weight(.semibold))
                .foregroundStyle(color)
            Text(label)
                .font(.caption)
                .foregroundStyle(.tertiary)
        }
    }
}
