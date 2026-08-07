import Foundation
@testable import Hce

nonisolated(unsafe) var passed = 0, failed = 0
func ok(_ c: Bool, _ m: String) { if c { passed += 1 } else { print("FAIL: \(m)"); failed += 1 } }
func eq<T: Equatable>(_ a: T, _ b: T, _ m: String) { if a == b { passed += 1 } else { print("FAIL: \(m) — \(a) != \(b)"); failed += 1 } }

let key = "test-key-32-bytes-for-swift-test!".data(using: .utf8)!
let uuid = Data([1, 149, 227, 160, 124, 46, 123, 65, 143, 61, 154, 108, 30, 11, 77, 39])

if let c = HceCodec(key: key, level: .universal, mode: .sealed) {
    if let enc = c.encode(uuid) { ok(!enc.isEmpty, "sealed encode")
        if let dec = c.decode(enc) { eq(dec, uuid, "sealed roundtrip") } }
}
if let c = HceCodec(key: nil, level: .universal, mode: .plain) {
    if let enc = c.encode(uuid), let dec = c.decode(enc) { eq(dec, uuid, "plain roundtrip") }
}
if let c = HceCodec(key: key, level: .universal, mode: .open) {
    if let enc = c.encode(uuid) { ok(enc.hasPrefix("B"), "open prefix")
        if let dec = c.decode(enc) { eq(dec, uuid, "open roundtrip") } }
}
if HceCodec(key: nil, level: .universal, mode: .sealed) == nil { passed += 1 } else { print("FAIL: sealed needs key"); failed += 1 }
for lv: HceCodec.Level in [.universal, .eu, .en, .numeric, .ko] {
    if let c = HceCodec(key: key, level: lv, mode: .sealed),
       let enc = c.encode(uuid), let dec = c.decode(enc) { eq(dec, uuid, "level \(lv)") }
}
var seen = Set<String>()
for lv: HceCodec.Level in [.universal, .eu, .en, .numeric, .ko] {
    if let c = HceCodec(key: key, level: lv, mode: .sealed), let enc = c.encode(uuid) { seen.insert(enc) }
}
eq(seen.count, 5, "5 levels different")
for bw: UInt32 in [16, 32, 48, 64, 80, 96, 112, 128] {
    if let c = HceCodec(key: key, level: .universal, mode: .sealed) {
        c.withBitWidth(bw); let bl = Int((bw+7)/8); var d = Data(count: bl)
        for i in 0..<bl { d[i] = UInt8((i*17+1)%256) }
        if let enc = c.encode(d), let dec = c.decode(enc) { eq(dec.suffix(bl), d, "bw=\(bw)") }
    }
}
if let c = HceCodec(key: key, level: .universal, mode: .sealed) {
    c.withCase(.lower)
    if let enc = c.encode(uuid), let dec = c.decode(enc) { ok(!enc.contains{$0.isUppercase}, "lower"); eq(dec, uuid, "lower rt") }
}
if let c = HceCodec(key: key, level: .universal, mode: .sealed) {
    c.withChunkNone()
    if let enc = c.encode(uuid) { ok(!enc.contains("-"), "chunk none") }
}
if let c1 = HceCodec(key: key, level: .universal, mode: .sealed),
   let c2 = HceCodec(key: key, level: .universal, mode: .sealed) {
    c2.withCipherShuffle()
    if let e1 = c1.encode(uuid), let e2 = c2.encode(uuid), let d2 = c2.decode(e2) { ok(e1 != e2, "shuffle diff"); eq(d2, uuid, "shuffle rt") }
}
if let c = HceCodec(key: key, level: .universal, mode: .sealed) {
    let data = Data(repeating: 0xFF, count: 16)
    if let enc = c.encode(data), let dec = c.decode(enc) { eq(dec, data, "max value") }
}
print("\(passed) passed, \(failed) failed")
exit(failed > 0 ? 1 : 0)
