// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michele Lombardi

import AppKit
import Combine
import Darwin
import SwiftUI

private final class QRYCapsulePanel: NSPanel {
    override var canBecomeKey: Bool { false }
    override var canBecomeMain: Bool { false }
    override func constrainFrameRect(_ frameRect: NSRect, to screen: NSScreen?) -> NSRect { frameRect }
}

private struct CapsuleContent: View {
    @ObservedObject var store: QRYBridgeStore
    let quit: () -> Void

    var body: some View {
        Button { QRYBridgeStore.open("today") } label: {
            HStack(spacing: 5) {
                QRYPipView(snapshot: store.snapshot, size: 25)
                if store.snapshot?.status == "monitoring" {
                    Text("\(store.snapshot?.displayedWpm ?? 0)")
                        .font(.system(size: 12, weight: .semibold, design: .rounded)).monospacedDigit()
                        .frame(width: 24, alignment: .trailing)
                    Text("WPM").font(.system(size: 8, weight: .medium)).foregroundStyle(.secondary)
                } else {
                    Text(store.statusText).font(.system(size: 10, weight: .medium))
                }
            }
            .padding(.horizontal, 7)
            .frame(width: 104, height: 30)
            .background(.ultraThinMaterial, in: RoundedRectangle(cornerRadius: 12))
            .overlay(RoundedRectangle(cornerRadius: 12).strokeBorder(Color.primary.opacity(0.12), lineWidth: 0.5))
            .contentShape(RoundedRectangle(cornerRadius: 12))
        }
        .buttonStyle(.plain)
        .help("QRY · click for Today; right-click for Statistics and Settings")
        .accessibilityLabel(store.snapshot?.status == "monitoring"
            ? "QRY, \(store.snapshot?.displayedWpm ?? 0) words per minute" : "QRY, \(store.statusText)")
        .contextMenu {
            Button("Today") { QRYBridgeStore.open("today") }
            Button("Statistics") { QRYBridgeStore.open("statistics") }
            Button("QRY Settings") { QRYBridgeStore.open("settings") }
            Divider()
            Button("Close capsule", action: quit)
        }
    }
}

private final class CapsuleController: NSObject, NSApplicationDelegate {
    private let store = QRYBridgeStore.shared
    private var panels: [UInt32: QRYCapsulePanel] = [:]
    private var subscription: AnyCancellable?
    private var visibilitySubscription: AnyCancellable?
    private var lastLayout = ""
    private var instanceLock: Int32 = -1

    func applicationDidFinishLaunching(_ notification: Notification) {
        let directory = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Caches/app.typepulse.desktop.capsule")
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true,
                                                attributes: [.posixPermissions: 0o700])
        instanceLock = Darwin.open(directory.appendingPathComponent("instance.lock").path, O_CREAT | O_RDWR, 0o600)
        guard instanceLock >= 0, flock(instanceLock, LOCK_EX | LOCK_NB) == 0 else {
            NSApp.terminate(nil); return
        }
        NSApp.setActivationPolicy(.accessory)
        subscription = store.$snapshot.sink { [weak self] _ in
            DispatchQueue.main.async { self?.update() }
        }
        visibilitySubscription = store.$capsuleEnabled.sink { [weak self] _ in
            DispatchQueue.main.async { self?.update() }
        }
        NotificationCenter.default.addObserver(self, selector: #selector(update),
            name: NSApplication.didChangeScreenParametersNotification, object: nil)
        NSWorkspace.shared.notificationCenter.addObserver(self, selector: #selector(update),
            name: NSWorkspace.didWakeNotification, object: nil)
        store.start()
        update()
    }

    @objc private func update() {
        // Missing status stays clickable so QRY can be started; an explicit opt-out hides it.
        if !store.capsuleEnabled {
            panels.values.forEach { $0.orderOut(nil) }
            writeLayout()
            return
        }
        var present = Set<UInt32>()
        for screen in NSScreen.screens {
            guard let number = screen.deviceDescription[NSDeviceDescriptionKey("NSScreenNumber")] as? NSNumber else { continue }
            let id = number.uint32Value
            present.insert(id)
            let panel: QRYCapsulePanel
            if let previous = panels[id] { panel = previous }
            else {
                panel = QRYCapsulePanel(contentRect: .zero, styleMask: [.borderless, .nonactivatingPanel], backing: .buffered, defer: false)
                panel.title = "QRY Capsule — \(screen.localizedName)"
                panel.isOpaque = false
                panel.backgroundColor = .clear
                panel.hasShadow = true
                panel.hidesOnDeactivate = false
                panel.isReleasedWhenClosed = false
                panel.level = .floating
                panel.collectionBehavior = [.canJoinAllSpaces, .stationary, .ignoresCycle, .fullScreenAuxiliary]
                panel.animationBehavior = .none
                panel.contentView = NSHostingView(rootView: CapsuleContent(store: store, quit: { NSApp.terminate(nil) }))
                panels[id] = panel
            }
            let width: CGFloat = 104
            let frame = NSRect(x: screen.frame.maxX - width - 12, y: screen.frame.maxY - 35,
                               width: width, height: 30)
            if panel.frame != frame { panel.setFrame(frame, display: true) }
            if !panel.isVisible { panel.orderFrontRegardless() }
        }
        for id in Array(panels.keys) where !present.contains(id) {
            panels.removeValue(forKey: id)?.close()
        }
        writeLayout()
    }

    private func writeLayout() {
        let windows: [[String: Any]] = panels.sorted { $0.key < $1.key }.map { id, panel in
            let screen = panel.screen?.frame ?? .zero
            return ["displayId": id, "x": panel.frame.minX, "y": panel.frame.minY,
             "width": panel.frame.width, "height": panel.frame.height, "visible": panel.isVisible,
             "screenX": screen.minX, "screenY": screen.minY, "screenWidth": screen.width, "screenHeight": screen.height]
        }
        guard let data = try? JSONSerialization.data(withJSONObject: windows, options: [.sortedKeys]),
              let signature = String(data: data, encoding: .utf8), signature != lastLayout else { return }
        lastLayout = signature
        let directory = FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Caches/app.typepulse.desktop.capsule")
        try? FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true,
                                                attributes: [.posixPermissions: 0o700])
        let path = directory.appendingPathComponent("layout-v1.json")
        try? data.write(to: path, options: .atomic)
        try? FileManager.default.setAttributes([.posixPermissions: 0o600], ofItemAtPath: path.path)
    }

    func applicationWillTerminate(_ notification: Notification) { store.stop() }
}

if CommandLine.arguments.contains("--self-test") {
    let fixture: [String: Any] = [
        "schemaVersion": 1, "generatedAtUnixMs": Date().timeIntervalSince1970 * 1000,
        "instanceId": "test", "sequence": 1, "status": "monitoring", "displayedWpm": 65,
        "animationBand": "fast", "celebrationSequence": 0, "capsuleEnabled": true,
        "externalEnabled": true, "notchPipEnabled": true,
        "presentation": ["visible": true, "behavior": "walk"],
        "today": ["date": "2026-10-28", "estimatedWords": 100.0, "peakWpm": 80.0, "activeTypingSeconds": 60.0]
    ]
    let data = try JSONSerialization.data(withJSONObject: fixture)
    precondition(QRYBridgeSnapshot.decode(data)?.displayedWpm == 65)
    precondition(QRYBridgeSnapshot.decode(data, now: Date().addingTimeInterval(31)) == nil)
    var future = fixture; future["schemaVersion"] = 2
    let futureData = try JSONSerialization.data(withJSONObject: future)
    precondition(QRYBridgeSnapshot.decode(futureData) == nil)
    var invalid = fixture; invalid["displayedWpm"] = -1
    let invalidData = try JSONSerialization.data(withJSONObject: invalid)
    precondition(QRYBridgeSnapshot.decode(invalidData) == nil)
    precondition(QRYBridgeSnapshot.decode(Data(repeating: 32, count: 65_537)) == nil)
    let day: [String: Any] = ["date": "2026-10-28", "estimatedWords": 100.0, "averageWpm": 60.0, "peakWpm": 80.0]
    let records: [String: Any] = ["peakWpm": 120.0, "sustained30Wpm": 90.0, "sustained60Wpm": 80.0]
    var statisticsFixture = fixture
    statisticsFixture["statistics"] = ["records": records, "days": [day]]
    let detailedData = try JSONSerialization.data(withJSONObject: statisticsFixture)
    precondition(QRYBridgeSnapshot.decode(detailedData)?.statistics?.records.peakWpm == 120)
    statisticsFixture["statistics"] = ["records": records, "days": [day, day]]
    let duplicateDaysData = try JSONSerialization.data(withJSONObject: statisticsFixture)
    precondition(QRYBridgeSnapshot.decode(duplicateDaysData) == nil)
    statisticsFixture["statistics"] = ["records": records, "days": Array(repeating: day, count: 91)]
    let excessiveDaysData = try JSONSerialization.data(withJSONObject: statisticsFixture)
    precondition(QRYBridgeSnapshot.decode(excessiveDaysData) == nil)
    var badDay = day; badDay["averageWpm"] = 100.0
    statisticsFixture["statistics"] = ["records": records, "days": [badDay]]
    let invalidAverageData = try JSONSerialization.data(withJSONObject: statisticsFixture)
    precondition(QRYBridgeSnapshot.decode(invalidAverageData) == nil)
    print("PASS: bridge decoding, freshness, schema, bounded history, records and invalid chart values")
    exit(0)
}

let app = NSApplication.shared
private let controller = CapsuleController()
app.delegate = controller
app.run()
