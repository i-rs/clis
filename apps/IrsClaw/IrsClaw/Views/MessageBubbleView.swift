import SwiftUI

struct MessageBubbleView: View {
    let message: AppMessage

    var body: some View {
        HStack {
            switch message {
            case .user(let text):
                Spacer(minLength: 60)
                VStack(alignment: .trailing, spacing: 2) {
                    Text(text)
                        .textSelection(.enabled)
                        .padding(12)
                        .background(Color.accentColor)
                        .foregroundColor(.white)
                        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }

            case .assistant(let text):
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 6) {
                        Image(systemName: "bolt.fill")
                            .font(.caption)
                            .foregroundColor(.accentColor)
                        Text("Claw")
                            .font(.caption)
                            .foregroundStyle(.secondary)
                    }
                    .padding(.leading, 4)

                    MarkdownTextView(text: text)
                        .padding(12)
                        .background(Color(nsColor: .controlBackgroundColor))
                        .clipShape(RoundedRectangle(cornerRadius: 12, style: .continuous))
                }
                Spacer(minLength: 60)

            case .toolCall(let name, _, let result):
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 4) {
                        Image(systemName: "wrench.adjustable")
                            .font(.caption)
                        Text(name)
                            .font(.caption)
                            .fontWeight(.medium)
                    }
                    .foregroundStyle(.secondary)

                    if !result.isEmpty {
                        Text(smartTruncate(result, maxLen: 200))
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .lineLimit(3)
                    }
                }
                .padding(10)
                .background(Color(nsColor: .controlBackgroundColor).opacity(0.5))
                .clipShape(RoundedRectangle(cornerRadius: 8))
                .overlay(
                    RoundedRectangle(cornerRadius: 8)
                        .stroke(Color.secondary.opacity(0.15), lineWidth: 1)
                )
                Spacer(minLength: 60)

            case .error(let text):
                HStack {
                    Image(systemName: "exclamationmark.triangle.fill")
                        .foregroundStyle(.red)
                    Text(text)
                        .font(.callout)
                        .foregroundStyle(.red)
                }
                .padding(10)
                .background(Color.red.opacity(0.08))
                .clipShape(RoundedRectangle(cornerRadius: 8))
                Spacer(minLength: 60)

            case .status(let text):
                HStack(spacing: 6) {
                    ProgressView()
                        .scaleEffect(0.6)
                    Text(text)
                        .font(.caption)
                        .foregroundStyle(.secondary)
                }
                .frame(maxWidth: .infinity, alignment: .center)
                .padding(.vertical, 4)

            case .reasoning(let text):
                VStack(alignment: .leading, spacing: 2) {
                    HStack(spacing: 4) {
                        Image(systemName: "brain")
                            .font(.caption2)
                        Text("Thinking")
                            .font(.caption2)
                            .fontWeight(.medium)
                    }
                    .foregroundStyle(.tertiary)

                    Text(text)
                        .font(.caption)
                        .foregroundStyle(.tertiary)
                        .lineLimit(5)
                }
                .padding(8)
                .background(Color.secondary.opacity(0.06))
                .clipShape(RoundedRectangle(cornerRadius: 8))
                Spacer(minLength: 60)
            }
        }
    }

    private func smartTruncate(_ text: String, maxLen: Int) -> String {
        if text.count <= maxLen { return text }
        return String(text.prefix(maxLen)) + "..."
    }
}

/// Simple markdown rendering for assistant messages.
struct MarkdownTextView: View {
    let text: String

    var body: some View {
        // Split by code blocks first
        let blocks = splitByCodeBlocks(text)

        VStack(alignment: .leading, spacing: 6) {
            ForEach(Array(blocks.enumerated()), id: \.offset) { _, block in
                if block.isCode {
                    CodeBlockView(code: block.content, language: block.language)
                } else {
                    InlineMarkdownView(text: block.content)
                }
            }
        }
    }

    private func splitByCodeBlocks(_ text: String) -> [(content: String, isCode: Bool, language: String)] {
        var result: [(String, Bool, String)] = []
        let lines = text.components(separatedBy: "\n")
        var inCodeBlock = false
        var codeLines: [String] = []
        var language = ""
        var inlineLines: [String] = []

        for line in lines {
            if line.hasPrefix("```") {
                if inCodeBlock {
                    // End code block
                    if !inlineLines.isEmpty {
                        result.append((inlineLines.joined(separator: "\n"), false, ""))
                        inlineLines.removeAll()
                    }
                    result.append((codeLines.joined(separator: "\n"), true, language))
                    codeLines.removeAll()
                    language = ""
                    inCodeBlock = false
                } else {
                    // Start code block
                    if !inlineLines.isEmpty {
                        result.append((inlineLines.joined(separator: "\n"), false, ""))
                        inlineLines.removeAll()
                    }
                    language = String(line.dropFirst(3)).trimmingCharacters(in: .whitespaces)
                    inCodeBlock = true
                }
            } else if inCodeBlock {
                codeLines.append(line)
            } else {
                inlineLines.append(line)
            }
        }

        // Flush remaining
        if !inlineLines.isEmpty {
            result.append((inlineLines.joined(separator: "\n"), false, ""))
        }
        if !codeLines.isEmpty {
            result.append((codeLines.joined(separator: "\n"), true, language))
        }

        return result
    }
}

struct CodeBlockView: View {
    let code: String
    let language: String

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            if !language.isEmpty {
                Text(language)
                    .font(.caption2)
                    .foregroundStyle(.tertiary)
                    .padding(.horizontal, 8)
                    .padding(.vertical, 4)
                    .background(Color.secondary.opacity(0.1))
            }

            ScrollView(.horizontal, showsIndicators: true) {
                Text(code)
                    .font(.system(.caption, design: .monospaced))
                    .foregroundStyle(.primary)
                    .padding(8)
                    .textSelection(.enabled)
            }
        }
        .background(Color(nsColor: .controlBackgroundColor).opacity(0.5))
        .clipShape(RoundedRectangle(cornerRadius: 8))
        .overlay(
            RoundedRectangle(cornerRadius: 8)
                .stroke(Color.secondary.opacity(0.15), lineWidth: 1)
        )
    }
}

struct InlineMarkdownView: View {
    let text: String

    var body: some View {
        // Simple inline rendering: handle bold, italic, inline code, links
        let segments = parseInlineMarkdown(text)

        Text(segments)
            .textSelection(.enabled)
            .fixedSize(horizontal: false, vertical: false)
    }

    private func parseInlineMarkdown(_ text: String) -> AttributedString {
        var attributed = AttributedString(text)

        // Bold: **text** or __text__
        if let regex = try? NSRegularExpression(pattern: "\\*\\*(.+?)\\*\\*|__(.+?)__") {
            let nsRange = NSRange(text.startIndex..., in: text)
            for match in regex.matches(in: text, range: nsRange).reversed() {
                let range = match.range
                if let swiftRange = Range(range, in: text) {
                    // Remove markers ** ** or __ __
                    let clean = String(text[swiftRange]).dropFirst(2).dropLast(2)
                    var cleanAttr = AttributedString(String(clean))
                    cleanAttr.font = .boldSystemFont(ofSize: NSFont.systemFontSize)
                    if let attrRange = Range(match.range(at: 1), in: text) ?? Range(match.range(at: 2), in: text) {
                        if let attributedRange = Range(attrRange, in: attributed) {
                            attributed.replaceSubrange(attributedRange, with: cleanAttr)
                        }
                    }
                }
            }
        }

        // Inline code: `text`
        if let regex = try? NSRegularExpression(pattern: "`(.+?)`") {
            let nsRange = NSRange(text.startIndex..., in: text)
            for match in regex.matches(in: text, range: nsRange).reversed() {
                let range = match.range(at: 1)
                if let swiftRange = Range(range, in: text) {
                    let codeText = String(text[swiftRange])
                    var attrText = AttributedString(codeText)
                    attrText.font = .monospacedSystemFont(ofSize: NSFont.systemFontSize - 1, weight: .regular)
                    attrText.backgroundColor = .init(red: 0.9, green: 0.9, blue: 0.9, alpha: 0.3)
                    if let attributedRange = Range(match.range, in: attributed) {
                        attributed.replaceSubrange(attributedRange, with: attrText)
                    }
                }
            }
        }

        return attributed
    }
}


