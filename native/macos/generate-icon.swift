import AppKit

let output = CommandLine.arguments.dropFirst().first ?? "OpenTerm-1024.png"
let size = NSSize(width: 1024, height: 1024)
let image = NSImage(size: size)

image.lockFocus()
let bounds = NSRect(origin: .zero, size: size)
let shape = NSBezierPath(roundedRect: bounds.insetBy(dx: 36, dy: 36), xRadius: 220, yRadius: 220)
let gradient = NSGradient(colors: [
    NSColor(calibratedRed: 0.03, green: 0.06, blue: 0.10, alpha: 1),
    NSColor(calibratedRed: 0.03, green: 0.18, blue: 0.25, alpha: 1),
])!
gradient.draw(in: shape, angle: -55)

let glow = NSBezierPath(ovalIn: NSRect(x: 160, y: 160, width: 704, height: 704))
NSColor(calibratedRed: 0.20, green: 0.82, blue: 0.96, alpha: 0.12).setFill()
glow.fill()

let mascot = [
    "      ▄▄      ",
    "▗▄▄▄▄▄██▄▄▄▄▄▖",
    "▐ ────────── ▌",
    "▐   ●    ●   ▌",
    "▐    ╰──╯    ▌",
    "▝▀▀▀▀▀▀▀▀▀▀▀▀▘",
    "  ▝▘      ▝▘  ",
]
let font = NSFont.monospacedSystemFont(ofSize: 58, weight: .bold)
let style = NSMutableParagraphStyle()
style.alignment = .center
let attributes: [NSAttributedString.Key: Any] = [
    .font: font,
    .foregroundColor: NSColor(calibratedRed: 0.49, green: 0.83, blue: 0.98, alpha: 1),
    .paragraphStyle: style,
]
for (index, line) in mascot.enumerated() {
    let y = 675 - CGFloat(index * 76)
    line.draw(in: NSRect(x: 80, y: y, width: 864, height: 82), withAttributes: attributes)
}
image.unlockFocus()

guard let tiff = image.tiffRepresentation,
      let bitmap = NSBitmapImageRep(data: tiff),
      let png = bitmap.representation(using: .png, properties: [:]) else {
    fatalError("Unable to render OpenTerm icon")
}
try png.write(to: URL(fileURLWithPath: output))
