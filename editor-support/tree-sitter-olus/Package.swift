// swift-tools-version:5.3
import PackageDescription

let package = Package(
    name: "TreeSitterOlus",
    products: [
        .library(name: "TreeSitterOlus", targets: ["TreeSitterOlus"]),
    ],
    dependencies: [
        .package(url: "https://github.com/ChimeHQ/SwiftTreeSitter", from: "0.8.0"),
    ],
    targets: [
        .target(
            name: "TreeSitterOlus",
            dependencies: [],
            path: ".",
            sources: [
                "src/parser.c",
                // NOTE: if your language has an external scanner, add it here.
            ],
            resources: [
                .copy("queries")
            ],
            publicHeadersPath: "bindings/swift",
            cSettings: [.headerSearchPath("src")]
        ),
        .testTarget(
            name: "TreeSitterOlusTests",
            dependencies: [
                "SwiftTreeSitter",
                "TreeSitterOlus",
            ],
            path: "bindings/swift/TreeSitterOlusTests"
        )
    ],
    cLanguageStandard: .c11
)
