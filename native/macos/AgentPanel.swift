import AppKit
import SwiftUI

struct AgentPanelView: View {
    @State private var question = ""
    @State private var answer = "Ask about your session, errors, or next command."
    @State private var busy = false

    var body: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack {
                Text("▐●‿●▌").font(.system(size: 22, design: .monospaced)).foregroundStyle(.cyan)
                VStack(alignment: .leading, spacing: 2) {
                    Text("Agent Panel").font(.title3.bold())
                    Text("Read-only · local-first").font(.caption).foregroundStyle(.secondary)
                }
                Spacer()
            }
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
        .padding(18)
        .frame(minWidth: 360, minHeight: 420)
        .background(LinearGradient(colors: [.black.opacity(0.92), .cyan.opacity(0.08)],
                                   startPoint: .topLeading, endPoint: .bottomTrailing))
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
                }
            } catch {
                DispatchQueue.main.async {
                    busy = false
                    answer = "Error: \(error.localizedDescription)"
                }
            }
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

@main
struct AgentPanelApp: App {
    var body: some Scene {
        WindowGroup("OpenTerm Agent") {
            AgentPanelView()
        }
        .windowResizability(.contentSize)
        .defaultPosition(.trailing)
        .commands { CommandGroup(replacing: .newItem) {} }
    }
}
