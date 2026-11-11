import Foundation

/// Runs one OpenTerm tool and delivers its merged stdout/stderr on the main queue as it arrives.
/// Mutable state is confined to the serial `state` queue.
final class ToolStream: @unchecked Sendable {
    private let process = Process()
    private let pipe = Pipe()
    private var pending = Data()
    private var exitStatus: Int32?
    private var reachedEOF = false
    private var cancelled = false
    private let state = DispatchQueue(label: "openterm.tool-stream")
    private let onText: (String) -> Void
    private let onExit: (Int32) -> Void

    /// Longest byte run that can be a truncated UTF-8 scalar.
    private static let maxPartialScalar = 3

    init(onText: @escaping (String) -> Void, onExit: @escaping (Int32) -> Void) {
        self.onText = onText
        self.onExit = onExit
    }

    func start(_ args: [String]) throws {
        guard let command = args.first, let binary = toolBinary(named: command) else {
            throw CocoaError(.fileNoSuchFile, userInfo: [NSLocalizedDescriptionKey: missingToolsMessage])
        }
        process.executableURL = URL(fileURLWithPath: binary)
        process.arguments = Array(args.dropFirst())
        process.standardOutput = pipe
        process.standardError = pipe
        process.standardInput = FileHandle.nullDevice
        pipe.fileHandleForReading.readabilityHandler = { [weak self] handle in
            let data = handle.availableData
            if data.isEmpty { handle.readabilityHandler = nil }
            guard let self else { return }
            self.state.async { self.receive(data) }
        }
        process.terminationHandler = { [weak self] task in
            guard let self else { return }
            self.state.async { self.exited(task.terminationStatus) }
        }
        try process.run()
    }

    /// Stops the child and silences any output still in flight.
    func stop() {
        state.async { self.cancelled = true }
        if process.isRunning { process.terminate() }
    }

    private func receive(_ data: Data) {
        guard !data.isEmpty else {
            reachedEOF = true
            finishIfDone()
            return
        }
        pending.append(data)
        guard let text = takeDecodable(), !cancelled else { return }
        DispatchQueue.main.async { self.onText(text) }
    }

    private func exited(_ status: Int32) {
        exitStatus = status
        finishIfDone()
    }

    private func finishIfDone() {
        guard reachedEOF, let status = exitStatus, !cancelled else { return }
        DispatchQueue.main.async { self.onExit(status) }
    }

    /// Decodes the longest valid UTF-8 prefix, keeping a split trailing scalar for the next chunk.
    private func takeDecodable() -> String? {
        for held in 0...min(Self.maxPartialScalar, pending.count) {
            let end = pending.count - held
            if let text = String(data: pending.prefix(end), encoding: .utf8) {
                pending = Data(pending.suffix(held))
                return text.isEmpty ? nil : text
            }
        }
        let text = String(decoding: pending, as: UTF8.self)
        pending.removeAll()
        return text
    }
}

private let ansiPattern = try? NSRegularExpression(
    pattern: "\u{1B}\\][^\u{07}\u{1B}]*(?:\u{07}|\u{1B}\\\\)|\u{1B}\\[[0-?]*[ -/]*[@-~]|\u{1B}[@-_]|\r"
)

/// Removes ANSI colour/cursor sequences and carriage returns for plain-text display.
func stripANSI(_ text: String) -> String {
    guard let ansiPattern else { return text }
    let range = NSRange(text.startIndex..., in: text)
    return ansiPattern.stringByReplacingMatches(in: text, range: range, withTemplate: "")
}

/// Publishes the HUD line, re-reading it whenever the file changes on disk.
final class HudMonitor: ObservableObject {
    @Published private(set) var line = "ai · not connected"

    private var source: DispatchSourceFileSystemObject?
    private var active = false

    /// How long to wait before retrying when neither the HUD file nor its directory exist yet.
    private static let retryDelay: TimeInterval = 5

    func start() {
        guard !active else { return }
        active = true
        reload()
        arm()
    }

    func stop() {
        active = false
        source?.cancel()
        source = nil
    }

    private func reload() {
        if let text = readHudLine(), !text.isEmpty { line = text }
    }

    /// Watches the file itself; until it exists, watches its directory for its creation.
    private func arm() {
        source?.cancel()
        source = nil
        guard active else { return }
        let file = hudURL
        let watchingFile = FileManager.default.fileExists(atPath: file.path)
        let target = watchingFile ? file : file.deletingLastPathComponent()
        let fd = open(target.path, O_EVTONLY)
        guard fd >= 0 else {
            DispatchQueue.main.asyncAfter(deadline: .now() + Self.retryDelay) { [weak self] in self?.arm() }
            return
        }
        let events: DispatchSource.FileSystemEvent = [.write, .extend, .delete, .rename]
        let watcher = DispatchSource.makeFileSystemObjectSource(fileDescriptor: fd, eventMask: events, queue: .main)
        watcher.setEventHandler { [weak self, weak watcher] in
            guard let self, let watcher else { return }
            self.reload()
            let replaced = !watcher.data.isDisjoint(with: [.delete, .rename])
            if replaced || !watchingFile { self.arm() }
        }
        watcher.setCancelHandler { close(fd) }
        source = watcher
        watcher.resume()
    }
}
