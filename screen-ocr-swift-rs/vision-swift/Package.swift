// swift-tools-version: 5.4
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "vision-swift",
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "vision-swift",
            type: .static,
            targets: ["vision-swift"]
        ),
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(name: "SwiftRs", url: "https://github.com/Brendonovich/swift-rs", from: "1.0.3"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "vision-swift",
            dependencies: [.product(name: "SwiftRs", package: "SwiftRs")]
        ),
        .testTarget(
            name: "vision-swiftTests",
            dependencies: ["vision-swift"]
        ),
    ]
)
