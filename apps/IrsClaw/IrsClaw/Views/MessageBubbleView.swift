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
                        .background(
                            LinearGradient(
                                colors: [Color.accentColor, Color.accentColor.opacity(0.8)],
                                startPoint: .topLeading,
                                endPoint: .bottomTrailing
                            )
                        )
                        .foregroundColor(.white)
                        .clipShape(MessageBubbleTail(isUser: true))
                }
                .transition(.move(edge: .trailing).combined(with: .opacity))

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
                        .background(Color.platformControlBackground)
                        .clipShape(MessageBubbleTail(isUser: false))
                }
                .transition(.move(edge: .leading).combined(with: .opacity))
                Spacer(minLength: 60)

            case .toolCall(let name, _, let result):
                VStack(alignment: .leading, spacing: 4) {
                    HStack(spacing: 4) {
                        Image(systemName: "wrench.adjustable")
                            .font(.caption)
                            .foregroundStyle(.accent)
                        Text(name)
                            .font(.caption)
                            .fontWeight(.medium)
                    }

                    if !result.isEmpty {
                        Text(smartTruncate(result, maxLen: 200))
                            .font(.caption)
                            .foregroundStyle(.tertiary)
                            .lineLimit(3)
                    }
                }
                .padding(10)
                .background(.ultraThinMaterial)
                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                .overlay(
                    RoundedRectangle(cornerRadius: 10)
                        .stroke(Color.accent.opacity(0.2), lineWidth: 1)
                )
                .transition(.scale.combined(with: .opacity))
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
                .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
                .transition(.scale.combined(with: .opacity))
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
                .background(.ultraThinMaterial)
                .clipShape(RoundedRectangle(cornerRadius: 8, style: .continuous))
                .transition(.slide.combined(with: .opacity))
                Spacer(minLength: 60)
            }
        }
    }

    private func smartTruncate(_ text: String, maxLen: Int) -> String {
        if text.count <= maxLen { return text }
        return String(text.prefix(maxLen)) + "..."
    }
}

/// Custom bubble shape with tail for chat messages
struct MessageBubbleTail: Shape {
    let isUser: Bool

    func path(in rect: CGRect) -> Path {
        let radius: CGFloat = 16
        let tailSize: CGFloat = 8

        var path = Path()

        if isUser {
            // User message: tail on bottom-right
            path.move(to: CGPoint(x: rect.minX + radius, y: rect.minY))
            path.addLine(to: CGPoint(x: rect.maxX - radius, y: rect.minY))
            path.addArc(center: CGPoint(x: rect.maxX - radius, y: rect.minY + radius),
                       radius: radius, startAngle: .degrees(270), endAngle: .degrees(0), clockwise: false)
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY - radius - tailSize))
            path.addArc(center: CGPoint(x: rect.maxX - radius, y: rect.maxY - radius - tailSize),
                       radius: radius, startAngle: .degrees(0), endAngle: .degrees(90), clockwise: false)
            path.addLine(to: CGPoint(x: rect.maxX - tailSize, y: rect.maxY))
            path.addLine(to: CGPoint(x: rect.maxX - tailSize - 2, y: rect.maxY - 2))
            path.addLine(to: CGPoint(x: rect.maxX - radius - tailSize, y: rect.maxY))
            path.addArc(center: CGPoint(x: rect.minX + radius, y: rect.maxY - radius),
                       radius: radius, startAngle: .degrees(90), endAngle: .degrees(180), clockwise: false)
            path.addLine(to: CGPoint(x: rect.minX, y: rect.minY + radius))
            path.addArc(center: CGPoint(x: rect.minX + radius, y: rect.minY + radius),
                       radius: radius, startAngle: .degrees(180), endAngle: .degrees(270), clockwise: false)
            path.closeSubpath()
        } else {
            // Assistant message: tail on bottom-left
            path.move(to: CGPoint(x: rect.minX + radius, y: rect.minY))
            path.addLine(to: CGPoint(x: rect.maxX - radius, y: rect.minY))
            path.addArc(center: CGPoint(x: rect.maxX - radius, y: rect.minY + radius),
                       radius: radius, startAngle: .degrees(270), endAngle: .degrees(0), clockwise: false)
            path.addLine(to: CGPoint(x: rect.maxX, y: rect.maxY - radius))
            path.addArc(center: CGPoint(x: rect.maxX - radius, y: rect.maxY - radius),
                       radius: radius, startAngle: .degrees(0), endAngle: .degrees(90), clockwise: false)
            path.addLine(to: CGPoint(x: rect.minX + radius + tailSize, y: rect.maxY))
            path.addArc(center: CGPoint(x: rect.minX + radius + tailSize, y: rect.maxY - radius),
                       radius: radius, startAngle: .degrees(90), endAngle: .degrees(180), clockwise: false)
            path.addLine(to: CGPoint(x: rect.minX + tailSize, y: rect.maxY - tailSize))
            path.addLine(to: CGPoint(x: rect.minX + tailSize + 2, y: rect.maxY - 2))
            path.addLine(to: CGPoint(x: rect.minX + tailSize, y: rect.maxY - radius))
            path.addArc(center: CGPoint(x: rect.minX + radius, y: rect.minY + radius),
                       radius: radius, startAngle: .degrees(180), endAngle: .degrees(270), clockwise: false)
            path.closeSubpath()
        }

        return path
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
        .background(.ultraThinMaterial)
        .clipShape(RoundedRectangle(cornerRadius: 10, style: .continuous))
        .overlay(
            RoundedRectangle(cornerRadius: 10)
                .stroke(Color.accent.opacity(0.15), lineWidth: 1)
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
                    #if os(macOS)
                    let boldFontSize = NSFont.systemFontSize
                    #else
                    let boldFontSize = UIFont.systemFontSize
                    #endif
                    cleanAttr.font = .boldSystemFont(ofSize: boldFontSize)
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
                    #if os(macOS)
                    let fontSize = NSFont.systemFontSize
                    #else
                    let fontSize = UIFont.systemFontSize
                    #endif
                    attrText.font = .monospacedSystemFont(ofSize: fontSize - 1, weight: .regular)
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


