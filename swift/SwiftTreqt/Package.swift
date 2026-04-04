// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "SwiftTreqt",
    platforms: [
        .macOS(.v13)
    ],
    products: [
        .library(
            name: "SwiftTreqt",
            targets: ["SwiftTreqt"]
        ),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "SwiftTreqt",
            dependencies: []
        ),
        .testTarget(
            name: "SwiftTreqtTests",
            dependencies: ["SwiftTreqt"]
        ),
    ]
)
