// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 Michele Lombardi

import AppKit
import Combine
import Foundation

struct QRYBridgeSnapshot: Decodable, Equatable {
    struct Records: Decodable, Equatable {
        let peakWpm: Double
        let sustained30Wpm: Double
        let sustained60Wpm: Double
    }
    struct Day: Decodable, Equatable {
        let date: String
        let estimatedWords: Double
        let averageWpm: Double
        let peakWpm: Double
    }
    struct Statistics: Decodable, Equatable {
        let records: Records
        let days: [Day]

        var isValid: Bool {
            guard days.count <= 90,
                  [records.peakWpm, records.sustained30Wpm, records.sustained60Wpm]
                    .allSatisfy({ $0.isFinite && $0 >= 0 }) else { return false }
            var previous = ""
            for day in days {
                guard day.date > previous, Self.validDate(day.date),
                      [day.estimatedWords, day.averageWpm, day.peakWpm].allSatisfy({ $0.isFinite && $0 >= 0 }),
                      day.averageWpm <= day.peakWpm else { return false }
                previous = day.date
            }
            return true
        }

        private static func validDate(_ value: String) -> Bool {
            let parts = value.split(separator: "-")
            guard value.count == 10, parts.count == 3,
                  let year = Int(parts[0]), let month = Int(parts[1]), let day = Int(parts[2]),
                  (1...9999).contains(year), (1...12).contains(month), (1...31).contains(day) else { return false }
            let calendar = Calendar(identifier: .gregorian)
            guard let date = calendar.date(from: DateComponents(year: year, month: month, day: day)) else { return false }
            let actual = calendar.dateComponents([.year, .month, .day], from: date)
            return actual.year == year && actual.month == month && actual.day == day
        }
    }
    struct Presentation: Decodable, Equatable { let visible: Bool; let behavior: String }
    struct Today: Decodable, Equatable {
        let date: String
        let estimatedWords: Double
        let peakWpm: Double
        let activeTypingSeconds: Double
    }
    let schemaVersion: Int
    let generatedAtUnixMs: Double
    let instanceId: String
    let sequence: UInt64
    let status: String
    let displayedWpm: Int
    let animationBand: String
    let celebrationSequence: UInt64
    let capsuleEnabled: Bool
    let externalEnabled: Bool
    let notchPipEnabled: Bool
    let presentation: Presentation
    let today: Today
    let statistics: Statistics?

    static let maximumBytes = 65_536
    static let maximumAge: TimeInterval = 30

    static func decode(_ data: Data, now: Date = Date()) -> QRYBridgeSnapshot? {
        guard data.count <= maximumBytes,
              let value = try? JSONDecoder().decode(Self.self, from: data),
              value.schemaVersion == 1,
              value.generatedAtUnixMs.isFinite,
              (-5...maximumAge).contains(now.timeIntervalSince1970 - value.generatedAtUnixMs / 1000),
              (0...999).contains(value.displayedWpm),
              ["monitoring", "paused", "permission_required"].contains(value.status),
              ["still", "steady", "fast", "intense"].contains(value.animationBand),
              ["breathe", "walk", "run", "tired"].contains(value.presentation.behavior),
              !value.instanceId.isEmpty, value.instanceId.count <= 128,
              value.today.date.count == 10,
              value.statistics?.isValid != false,
              value.statistics?.days.last.map({ $0.date <= value.today.date }) != false,
              [value.today.estimatedWords, value.today.peakWpm, value.today.activeTypingSeconds]
                .allSatisfy({ $0.isFinite && $0 >= 0 })
        else { return nil }
        return value
    }
}

final class QRYBridgeStore: ObservableObject {
    static let shared = QRYBridgeStore()
    static var defaultURL: URL {
        FileManager.default.homeDirectoryForCurrentUser
            .appendingPathComponent("Library/Caches/app.typepulse.desktop/integrations/state-v1.json")
    }

    @Published private(set) var snapshot: QRYBridgeSnapshot?
    @Published private(set) var capsuleEnabled = true
    private var timer: DispatchSourceTimer?
    private let queue = DispatchQueue(label: "qry.aggregate-reader", qos: .utility)
    private let url: URL

    init(url: URL = QRYBridgeStore.defaultURL) { self.url = url }

    func start() {
        guard timer == nil else { return }
        let source = DispatchSource.makeTimerSource(queue: queue)
        source.schedule(deadline: .now(), repeating: .milliseconds(500), leeway: .milliseconds(100))
        source.setEventHandler { [weak self] in
            guard let self else { return }
            let value: QRYBridgeSnapshot?
            if let handle = try? FileHandle(forReadingFrom: self.url) {
                defer { try? handle.close() }
                value = (try? handle.read(upToCount: QRYBridgeSnapshot.maximumBytes + 1))
                    .flatMap { QRYBridgeSnapshot.decode($0) }
            } else { value = nil }
            let preferences = self.url.deletingLastPathComponent().appendingPathComponent("preferences-v1.json")
            let control = (try? Data(contentsOf: preferences)).flatMap {
                try? JSONSerialization.jsonObject(with: $0) as? [String: Bool]
            }
            let capsuleEnabled = control?["capsuleEnabled"] ?? true
            DispatchQueue.main.async { [weak self] in
                guard let self else { return }
                if self.snapshot != value { self.snapshot = value }
                if self.capsuleEnabled != capsuleEnabled { self.capsuleEnabled = capsuleEnabled }
            }
        }
        timer = source
        source.resume()
    }

    func stop() {
        timer?.cancel()
        timer = nil
        snapshot = nil
    }

    var statusText: String {
        switch snapshot?.status {
        case "monitoring": return "QRY"
        case "paused": return "Paused"
        case "permission_required": return "Open QRY"
        default: return "Start QRY"
        }
    }

    static func open(_ page: String) {
        guard ["today", "statistics", "settings"].contains(page),
              let url = URL(string: "qry://open/\(page)") else { return }
        // Target the installed QRY explicitly even when development bundles coexist.
        let installed = URL(fileURLWithPath: "/Applications/QRY.app")
        if FileManager.default.fileExists(atPath: installed.path) {
            NSWorkspace.shared.open([url], withApplicationAt: installed,
                                    configuration: NSWorkspace.OpenConfiguration())
        } else { NSWorkspace.shared.open(url) }
    }
}
