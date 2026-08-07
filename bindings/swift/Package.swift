// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "Hce",
    platforms: [.macOS(.v14)],
    products: [
        .library(name: "Hce", targets: ["Hce"]),
    ],
    targets: [
        .target(
            name: "Hce",
            linkerSettings: [
                .linkedLibrary("hce_ffi"),
                .unsafeFlags(["-L../../target/release"]),
            ]
        ),
        .testTarget(name: "HceTests", dependencies: ["Hce"]),
    ]
)
