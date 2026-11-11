import AppKit
import CoreGraphics
import SwiftUI

struct AgentPanelView: View {
    @State private var question = ""
    @State private var answer = "Ask about your session, errors, or next command."
    @State private var raw = ""
    @State private var stream: ToolStream?
    @StateObject private var hud = HudMonitor()

    private var busy: Bool { stream != nil }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("▐●‿●▌").font(.system(size: 22, design: .monospaced)).foregroundStyle(.cyan)
                VStack(alignment: .leading, spacing: 2) {
                    Text("Agent Panel").font(.title3.bold())
                    Text("Read-only · does not steal shell focus").font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
            }
            Text(hud.line)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.cyan)
                .padding(8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.cyan.opacity(0.08), in: RoundedRectangle(cornerRadius: 8))
            TextField("Ask OpenTerm…", text: $question)
                .textFieldStyle(.roundedBorder)
                .onSubmit { ask() }
            controls
            ScrollView {
                Text(answer).font(.system(.body, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .textSelection(.enabled)
            }
            .padding(10)
            .background(Color.black.opacity(0.25), in: RoundedRectangle(cornerRadius: 12))
        }
        .padding(16)
        .frame(minWidth: 360, minHeight: 420)
        .background(LinearGradient(colors: [.black.opacity(0.92), .cyan.opacity(0.08)],
                                   startPoint: .topLeading, endPoint: .bottomTrailing))
        .onAppear { hud.start() }
        .onDisappear { hud.stop(); stop() }
    }

    private var controls: some View {
        HStack {
            Button("Ask") { ask() }.buttonStyle(.borderedProminent).tint(.cyan).disabled(busy)
            Button("Workspace") { runTool(["ai-workspace"]) }.disabled(busy)
            Button("Gain") { runTool(["ai-gain"]) }.disabled(busy)
            Spacer()
            if busy {
                ProgressView().controlSize(.small)
                Button("Stop") { stop() }
            }
        }
    }

    private func ask() {
        let trimmed = question.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty, !busy else { return }
        runTool(["ai-ask", "--workspace", trimmed])
    }

    private func runTool(_ args: [String]) {
        raw = ""
        answer = ""
        let tool = ToolStream(onText: { append($0) }, onExit: { finish($0) })
        do {
            try tool.start(args)
            stream = tool
        } catch {
            answer = "Error: \(error.localizedDescription)"
        }
    }

    private func append(_ chunk: String) {
        guard busy else { return }
        raw += chunk
        answer = stripANSI(raw)
    }

    private func finish(_ status: Int32) {
        stream = nil
        if answer.isEmpty { answer = status == 0 ? "No output." : "Exited with code \(status)." }
    }

    private func stop() {
        guard let stream else { return }
        stream.stop()
        self.stream = nil
        answer += answer.isEmpty ? "Stopped." : "\n\n[stopped]"
    }
}

@main
@MainActor
enum AgentPanelApp {
    static func main() {
        let app = NSApplication.shared
        app.setActivationPolicy(.accessory)
        let panel = makePanel()
        dock(panel)
        panel.orderFrontRegardless()
        app.run()
    }

    private static func makePanel() -> NSPanel {
        let panel = NSPanel(
            contentRect: NSRect(x: 0, y: 0, width: 380, height: 520),
            styleMask: [.titled, .closable, .resizable, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        panel.title = "OpenTerm Agent"
        panel.isFloatingPanel = true
        panel.hidesOnDeactivate = false
        panel.level = .floating
        panel.collectionBehavior = [.moveToActiveSpace, .fullScreenAuxiliary]
        panel.contentView = NSHostingView(rootView: AgentPanelView())
        return panel
    }
}

@MainActor
func dock(_ panel: NSPanel) {
    let options: CGWindowListOption = [.optionOnScreenOnly, .excludeDesktopElements]
    guard let raw = CGWindowListCopyWindowInfo(options, kCGNullWindowID) as? [[String: Any]] else {
        panel.center()
        return
    }
    let frames = raw.compactMap { info -> CGRect? in
        let owner = info[kCGWindowOwnerName as String] as? String ?? ""
        guard owner == "OpenTerm" else { return nil }
        guard let bounds = info[kCGWindowBounds as String] as? NSDictionary else { return nil }
        return CGRect(dictionaryRepresentation: bounds)
    }
    guard let term = frames.max(by: { $0.width * $0.height < $1.width * $1.height }) else {
        panel.center()
        return
    }
    var frame = panel.frame
    frame.size = NSSize(width: 380, height: min(520, max(420, term.height - 80)))
    frame.origin.x = term.maxX - frame.width - 16
    frame.origin.y = term.minY + 28
    panel.setFrame(frame, display: true)
}
