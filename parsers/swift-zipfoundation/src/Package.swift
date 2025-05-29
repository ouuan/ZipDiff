// swift-tools-version: 5.10

import PackageDescription

let package = Package(
    name: "unzip",
    dependencies: [
        .package(url: "https://github.com/weichsel/ZIPFoundation.git", exact: "0.9.19"),
    ],
    targets: [
        .executableTarget(name: "unzip", dependencies: ["ZIPFoundation"]),
    ]
)
