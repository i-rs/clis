import SwiftUI

struct SkillsPanel: View {
    @ObservedObject var service: ClawService
    @State private var searchText = ""
    @State private var expanded = Set<String>()

    var filteredSkills: [SkillInfo] {
        guard !searchText.isEmpty else { return service.skills }
        return service.skills.filter {
            $0.name.localizedCaseInsensitiveContains(searchText) ||
            $0.description.localizedCaseInsensitiveContains(searchText)
        }
    }

    var body: some View {
        VStack(spacing: 0) {
            if service.skills.isEmpty {
                Spacer()
                VStack(spacing: 8) {
                    Image(systemName: "book")
                        .font(.system(size: 32))
                        .foregroundStyle(.secondary)
                    Text("No skills found")
                        .foregroundStyle(.secondary)
                    Text("Add .md skill files to ~/.i-rs-claw/claw/skills/")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
                .frame(maxWidth: .infinity)
                Spacer()
            } else {
                List {
                    ForEach(filteredSkills) { skill in
                        SkillRow(skill: skill, isExpanded: expanded.contains(skill.name)) {
                            if expanded.contains(skill.name) {
                                expanded.remove(skill.name)
                            } else {
                                expanded.insert(skill.name)
                            }
                        }
                    }
                }
                .listStyle(.inset)
            }
        }
        .searchable(text: $searchText, prompt: "Search skills")
        .onAppear {
            Task { await service.fetchSkills() }
        }
    }
}

struct SkillRow: View {
    let skill: SkillInfo
    let isExpanded: Bool
    let onToggle: () -> Void

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Button(action: onToggle) {
                HStack(spacing: 8) {
                    Image(systemName: "book")
                        .foregroundStyle(.blue)
                        .font(.title3)
                        .frame(width: 24)

                    VStack(alignment: .leading, spacing: 2) {
                        Text(skill.name)
                            .font(.body)
                            .fontWeight(.medium)
                        if !skill.description.isEmpty && skill.description != skill.name {
                            Text(skill.description)
                                .font(.caption)
                                .foregroundStyle(.secondary)
                                .lineLimit(1)
                        }
                    }

                    Spacer()

                    Image(systemName: isExpanded ? "chevron.down" : "chevron.right")
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                }
                .padding(.vertical, 4)
                .contentShape(Rectangle())
            }
            .buttonStyle(.plain)

            if isExpanded {
                ScrollView {
                    Text(skill.content)
                        .font(.caption.monospaced())
                        .foregroundStyle(.secondary)
                        .textSelection(.enabled)
                        .padding(8)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(.ultraThinMaterial)
                        .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                }
                .frame(maxHeight: 300)
                .padding(.top, 4)
            }
        }
    }
}
