import Foundation

/// Outcome of a finished OpenTerm tool invocation.
struct ToolResult {
    static let launchFailure: Int32 = -1

    let status: Int32
    let stdout: String
    let stderr: String

    var ok: Bool { status == 0 }

    /// The most useful human-readable explanation of a failure.
    var errorText: String {
        let err = stderr.trimmingCharacters(in: .whitespacesAndNewlines)
        if !err.isEmpty { return err }
        let out = stdout.trimmingCharacters(in: .whitespacesAndNewlines)
        return out.isEmpty ? "Command failed with exit code \(status)." : out
    }
}

let missingToolsMessage = "OpenTerm tools not found. Launch from OpenTerm.app."

/// Locates an OpenTerm multicall binary inside the app bundle or `OPENTERM_RESOURCES`.
func toolBinary(named command: String) -> String? {
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

/// Runs `args` (first element is the tool name) off the main thread.
/// `input` is written to the child's stdin, which keeps secrets out of argv.
func runTool(_ args: [String], input: String? = nil) async -> ToolResult {
    await withCheckedContinuation { continuation in
        DispatchQueue.global(qos: .userInitiated).async {
            continuation.resume(returning: runToolSync(args, input: input))
        }
    }
}

private func runToolSync(_ args: [String], input: String?) -> ToolResult {
    guard let command = args.first, let binary = toolBinary(named: command) else {
        return ToolResult(status: ToolResult.launchFailure, stdout: "", stderr: missingToolsMessage)
    }
    let task = Process()
    task.executableURL = URL(fileURLWithPath: binary)
    task.arguments = Array(args.dropFirst())
    let out = Pipe(), err = Pipe(), stdin = Pipe()
    task.standardOutput = out
    task.standardError = err
    task.standardInput = input == nil ? FileHandle.nullDevice : stdin
    do {
        try task.run()
    } catch {
        return ToolResult(status: ToolResult.launchFailure, stdout: "",
                          stderr: error.localizedDescription)
    }
    if let input { feed(stdin, input) }
    let (outData, errData) = drain(out, err)
    task.waitUntilExit()
    return ToolResult(status: task.terminationStatus,
                      stdout: String(decoding: outData, as: UTF8.self),
                      stderr: String(decoding: errData, as: UTF8.self))
}

private func feed(_ stdin: Pipe, _ input: String) {
    // A child that exits early (e.g. unknown flag) must not kill us with SIGPIPE.
    signal(SIGPIPE, SIG_IGN)
    try? stdin.fileHandleForWriting.write(contentsOf: Data(input.utf8))
    try? stdin.fileHandleForWriting.close()
}

/// Reads both pipes concurrently so a chatty stderr can never deadlock stdout.
private func drain(_ out: Pipe, _ err: Pipe) -> (Data, Data) {
    var errData = Data()
    let group = DispatchGroup()
    group.enter()
    DispatchQueue.global(qos: .userInitiated).async {
        errData = err.fileHandleForReading.readDataToEndOfFile()
        group.leave()
    }
    let outData = out.fileHandleForReading.readDataToEndOfFile()
    group.wait()
    return (outData, errData)
}

/// The HUD line written by `ai-ask` (mirrors the Rust `data_root`).
var hudURL: URL {
    let dir = ProcessInfo.processInfo.environment["OPENTERM_DATA_DIR"]
        ?? FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Application Support/OpenTerm").path
    return URL(fileURLWithPath: dir).appendingPathComponent("hud")
}

func readHudLine() -> String? {
    (try? String(contentsOf: hudURL, encoding: .utf8))?
        .trimmingCharacters(in: .whitespacesAndNewlines)
}
