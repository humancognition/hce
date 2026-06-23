import Foundation
import Testing
@testable import Hce

@Test func sealedRoundtrip() {
    let key = "test-key-32-bytes-for-swift-test!".data(using: .utf8)!
    let uuid = Data([1, 149, 227, 160, 124, 46, 123, 65, 143, 61, 154, 108, 30, 11, 77, 39])

    guard let c = HceCodec(key: key, level: .universal, mode: .sealed) else {
        #expect(Bool(false), "failed to create codec")
        return
    }

    guard let encoded = c.encode(uuid) else {
        #expect(Bool(false), "encode failed")
        return
    }
    #expect(!encoded.isEmpty)

    guard let decoded = c.decode(encoded) else {
        #expect(Bool(false), "decode failed")
        return
    }
    #expect(decoded == uuid)
}
