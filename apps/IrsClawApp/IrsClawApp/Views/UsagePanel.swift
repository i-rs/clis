import SwiftUI

struct UsagePanel: View {
    @ObservedObject var service: ClawService
    @State private var selectedPeriod = "all"

    var body: some View {
        let total = service.totalTokenUsage
        List {
            Section {
                VStack(spacing: 16) {
                    if service.isLoadingStats {
                        ProgressView()
                            .frame(maxWidth: .infinity)
                            .padding()
                    } else if let stats = service.stats {
                        statsHeaderView(stats: stats)
                    } else {
                        totalRow(icon: "number", label: "Total Tokens", value: "\(total.totalTokens)")
                            .font(.title2.weight(.semibold))
                    }

                    HStack(spacing: 24) {
                        statItem(label: "Prompt", value: "\(total.promptTokens ?? 0)", color: .blue)
                        statItem(label: "Completion", value: "\(total.completionTokens ?? 0)", color: .green)
                        statItem(label: "Sessions", value: "\(service.sessionTokenUsage.count)", color: .orange)
                    }
                }
                .padding(.vertical, 8)
                .frame(maxWidth: .infinity)
            }

            Section {
                Picker("Period", selection: $selectedPeriod) {
                    Text("Today").tag("today")
                    Text("7 Days").tag("7d")
                    Text("30 Days").tag("30d")
                    Text("All").tag("all")
                }
                .pickerStyle(.segmented)
                .onChange(of: selectedPeriod) { _, newValue in
                    Task { await service.fetchStats(period: newValue) }
                }
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

            if service.sessionTokenUsage.isEmpty && !service.isLoadingStats && service.stats == nil {
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
        .listStyle(.insetGrouped)
        .task {
            await service.fetchStats(period: selectedPeriod)
        }
    }

    @ViewBuilder
    private func statsHeaderView(stats: StatsResponse) -> some View {
        VStack(spacing: 8) {
            if let totalTokens = stats.totalTokens {
                totalRow(icon: "number", label: "Total Tokens", value: "\(totalTokens)")
                    .font(.title2.weight(.semibold))
            }
            if let cost = stats.totalCostUsd {
                HStack(spacing: 8) {
                    Image(systemName: "dollarsign.circle")
                        .foregroundStyle(.green)
                    Text("Est. Cost")
                    Text(String(format: "$%.4f", cost))
                        .monospacedDigit()
                }
                .font(.callout)
            }
            if let today = stats.today {
                Divider()
                HStack(spacing: 16) {
                    if let req = today.requests {
                        VStack {
                            Text("\(req)")
                                .font(.headline.monospacedDigit())
                            Text("Requests")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                    if let tokens = today.tokens {
                        VStack {
                            Text("\(tokens)")
                                .font(.headline.monospacedDigit())
                            Text("Today Tokens")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                    if let cost = today.costUsd {
                        VStack {
                            Text(String(format: "$%.4f", cost))
                                .font(.headline.monospacedDigit())
                            Text("Today Cost")
                                .font(.caption2)
                                .foregroundStyle(.secondary)
                        }
                    }
                }
            }
        }
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
