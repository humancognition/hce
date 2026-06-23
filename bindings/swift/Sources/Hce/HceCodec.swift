import Foundation

@_silgen_name("hce_new")
func hce_new(_ key: UnsafePointer<UInt8>?, _ keyLen: Int, _ level: UInt8, _ mode: UInt8) -> OpaquePointer?

@_silgen_name("hce_destroy")
func hce_destroy(_ h: OpaquePointer?)

@_silgen_name("hce_with_bit_width")
func hce_with_bit_width(_ h: OpaquePointer?, _ bits: UInt32)

@_silgen_name("hce_with_domain_modulus")
func hce_with_domain_modulus(_ h: OpaquePointer?, _ hi: UInt64, _ lo: UInt64)

@_silgen_name("hce_with_cipher_kind")
func hce_with_cipher_kind(_ h: OpaquePointer?, _ kind: UInt8, _ key: UnsafePointer<UInt8>?, _ keyLen: Int)

@_silgen_name("hce_with_case")
func hce_with_case(_ h: OpaquePointer?, _ caseVal: UInt8)

@_silgen_name("hce_with_check_syllables")
func hce_with_check_syllables(_ h: OpaquePointer?, _ n: Int)

@_silgen_name("hce_with_separator")
func hce_with_separator(_ h: OpaquePointer?, _ sep: UInt8)

@_silgen_name("hce_with_chunk_none")
func hce_with_chunk_none(_ h: OpaquePointer?)

@_silgen_name("hce_with_chunk_fixed")
func hce_with_chunk_fixed(_ h: OpaquePointer?, _ charSize: Int)

@_silgen_name("hce_with_chunk_pattern")
func hce_with_chunk_pattern(_ h: OpaquePointer?, _ pattern: UnsafePointer<Int>?, _ count: Int)

@_silgen_name("hce_with_timestamp_config")
func hce_with_timestamp_config(_ h: OpaquePointer?, _ epochMs: Int64, _ gran: UInt8)

@_silgen_name("hce_encode")
func hce_encode(_ h: OpaquePointer?, _ data: UnsafePointer<UInt8>?, _ len: Int) -> HceResult

@_silgen_name("hce_decode")
func hce_decode(_ h: OpaquePointer?, _ input: UnsafePointer<UInt8>?, _ len: Int) -> HceResult

@_silgen_name("hce_recover")
func hce_recover(_ h: OpaquePointer?, _ input: UnsafePointer<UInt8>?, _ len: Int) -> RecoveryResult

@_silgen_name("hce_free_result")
func hce_free_result(_ r: HceResult)

@_silgen_name("hce_free_recovery_result")
func hce_free_recovery_result(_ r: RecoveryResult)

struct HceResult {
    var ok: Bool
    var data: UnsafePointer<UInt8>?
    var len: Int
    var errCode: Int32
}

struct RecoveryResult {
    var ok: Bool
    var corrected: UnsafePointer<UInt8>?
    var correctedLen: Int
    var candidateCount: Int
    var errCode: Int32
}

public final class HceCodec {
    private var ptr: OpaquePointer?

    public enum Level: UInt8 { case universal = 0, eu = 1, en = 2, numeric = 3 }
    public enum Mode: UInt8  { case sealed = 0, open = 1, plain = 2 }
    public enum CipherCase: UInt8 { case lower = 0, upper = 1 }
    public enum Granularity: UInt8 { case second = 0, minute = 1, hour = 2, day = 3, week = 4, month = 5 }

    public init?(key: Data?, level: Level, mode: Mode) {
        if mode != .plain && (key == nil || key!.isEmpty) { return nil }
        let kp = key?.withUnsafeBytes { $0.baseAddress?.assumingMemoryBound(to: UInt8.self) }
        guard let h = hce_new(kp, key?.count ?? 0, level.rawValue, mode.rawValue) else { return nil }
        ptr = h
    }

    deinit { if let p = ptr { hce_destroy(p) } }

    @discardableResult public func withBitWidth(_ bits: UInt32) -> Self { hce_with_bit_width(ptr, bits); return self }
    @discardableResult public func withModulus(_ m: UInt64) -> Self { hce_with_domain_modulus(ptr, UInt64(m >> 32), UInt64(m & 0xFFFF_FFFF)); return self }
    @discardableResult public func withCipherFeistel(_ key: Data?) -> Self {
        let kp = key?.withUnsafeBytes { $0.baseAddress?.assumingMemoryBound(to: UInt8.self) }
        hce_with_cipher_kind(ptr, 0, kp, key?.count ?? 0); return self
    }
    @discardableResult public func withCipherShuffle() -> Self { hce_with_cipher_kind(ptr, 1, nil, 0); return self }
    @discardableResult public func withCase(_ c: CipherCase) -> Self { hce_with_case(ptr, c.rawValue); return self }
    @discardableResult public func withCheckSyllables(_ n: Int) -> Self { hce_with_check_syllables(ptr, n); return self }
    @discardableResult public func withSeparator(_ ch: Character) -> Self { hce_with_separator(ptr, UInt8(ch.asciiValue ?? 45)); return self }
    @discardableResult public func withChunkNone() -> Self { hce_with_chunk_none(ptr); return self }
    @discardableResult public func withChunkFixed(_ n: Int) -> Self { hce_with_chunk_fixed(ptr, n); return self }
    @discardableResult public func withChunkPattern(_ pattern: [Int]) -> Self {
        let p = pattern; p.withUnsafeBufferPointer { hce_with_chunk_pattern(ptr, $0.baseAddress, $0.count) }; return self
    }
    @discardableResult public func withTimestampConfig(_ epochMs: Int64, _ gran: Granularity) -> Self { hce_with_timestamp_config(ptr, epochMs, gran.rawValue); return self }

    public func encode(_ data: Data) -> String? {
        let r = data.withUnsafeBytes { b in hce_encode(ptr, b.baseAddress?.assumingMemoryBound(to: UInt8.self), data.count) }
        guard r.ok, let d = r.data else { return nil }
        defer { hce_free_result(r) }
        return String(bytes: UnsafeBufferPointer(start: d, count: r.len), encoding: .utf8)
    }

    public func decode(_ input: String) -> Data? {
        let bytes = [UInt8](input.utf8)
        let r = bytes.withUnsafeBufferPointer { hce_decode(ptr, $0.baseAddress, input.utf8.count) }
        guard r.ok, let d = r.data else { return nil }
        defer { hce_free_result(r) }
        return Data(bytes: d, count: r.len)
    }

    public func recover(_ input: String) -> Data? {
        let bytes = [UInt8](input.utf8)
        let r = bytes.withUnsafeBufferPointer { hce_recover(ptr, $0.baseAddress, input.utf8.count) }
        defer { hce_free_recovery_result(r) }
        guard r.ok else { return nil }
        if r.correctedLen > 0, let c = r.corrected { return Data(bytes: c, count: r.correctedLen) }
        return decode(input)
    }
}
