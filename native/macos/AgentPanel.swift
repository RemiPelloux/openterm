import AppKit
import CoreGraphics
import SwiftUI

struct AgentPanelView: View {
    @State private var question = ""
    @State private var answer = "Ask about your session, errors, or next command."
    @State private var busy = false
    @State private var hud = "ai · not connected"

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
            Text(hud)
                .font(.system(.caption, design: .monospaced))
                .foregroundStyle(.cyan)
                .padding(8)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(Color.cyan.opacity(0.08), in: RoundedRectangle(cornerRadius: 8))
            TextField("Ask OpenTerm…", text: $question)
                .textFieldStyle(.roundedBorder)
                .onSubmit { ask() }
            HStack {
                Button("Ask") { ask() }.buttonStyle(.borderedProminent).tint(.cyan)
                Button("Workspace") { runTool(["ai-workspace"]) }
                Button("Gain") { runTool(["ai-gain"]) }
                Spacer()
                if busy { ProgressView().controlSize(.small) }
            }
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
        .onAppear { refreshHud() }
        .onReceive(Timer.publish(every: 1.5, on: .main, in: .common).autoconnect()) { _ in
            refreshHud()
        }
    }

    private func ask() {
        let trimmed = question.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        runTool(["ai-ask", "--workspace", trimmed])
    }

    private func runTool(_ args: [String]) {
        guard let command = args.first, let binary = toolBinary(named: command) else {
            answer = "OpenTerm tools not found. Launch from OpenTerm.app."
            return
        }
        busy = true
        DispatchQueue.global(qos: .userInitiated).async {
            let task = Process()
            task.executableURL = URL(fileURLWithPath: binary)
            task.arguments = Array(args.dropFirst())
            let pipe = Pipe()
            task.standardOutput = pipe
            task.standardError = pipe
            do {
                try task.run()
                task.waitUntilExit()
                let data = pipe.fileHandleForReading.readDataToEndOfFile()
                let text = String(data: data, encoding: .utf8) ?? ""
                DispatchQueue.main.async {
                    busy = false
                    answer = text.isEmpty ? "No output." : text
                    refreshHud()
                }
            } catch {
                DispatchQueue.main.async {
                    busy = false
                    answer = "Error: \(error.localizedDescription)"
                }
            }
        }
    }

    private func refreshHud() {
        let dir = ProcessInfo.processInfo.environment["OPENTERM_DATA_DIR"]
            ?? FileManager.default.homeDirectoryForCurrentUser
                .appendingPathComponent("Library/Application Support/OpenTerm").path
        let url = URL(fileURLWithPath: dir).appendingPathComponent("hud")
        if let line = try? String(contentsOf: url, encoding: .utf8) {
            hud = line.trimmingCharacters(in: .whitespacesAndNewlines)
        }
    }

    private func toolBinary(named command: String) -> String? {
        let dirs = [
            ProcessInfo.processInfo.environment["OPENTERM_RESOURCES"].map { "\($0)/bin" },
            Bundle.main.bundlePath + "/Contents/Resources/openterm/bin",
        ].compactMap { $0 }
        for dir in dirs {
            let path = "\(dir)/\(command)"
            if FileManager.default.isExecutableFile(atPath: path) { return path }
        }
        return nil
    }
}

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

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
dock(panel)
panel.orderFrontRegardless()
app.run()

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
