// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "FrfClient",
    platforms: [
        .iOS(.v16),
        .macOS(.v13),
    ],
    products: [
        .library(
            name: "FrfClient",
            targets: ["FrfClient"]
        ),
    ],
    targets: [
        // Prebuilt XCFramework produced by build_xcframework.sh
        .binaryTarget(
            name: "FrfClientFFI",
            path: "Sources/FrfClient/FrfClientFFI.xcframework"
        ),
        // UniFFI-generated Swift glue + hand-authored convenience wrappers
        .target(
            name: "FrfClient",
            dependencies: [.target(name: "FrfClientFFI")],
            path: "Sources/FrfClient",
            sources: ["frf.swift"],
            swiftSettings: [
                .unsafeFlags(["-suppress-warnings"])
            ]
        ),
    ]
)
