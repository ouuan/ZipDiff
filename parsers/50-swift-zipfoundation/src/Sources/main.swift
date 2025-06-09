import ZIPFoundation
import Foundation

let fileManager = FileManager()
var sourceURL = URL(fileURLWithPath: CommandLine.arguments[1])
var destinationURL = URL(fileURLWithPath: CommandLine.arguments[2])
do {
    try fileManager.unzipItem(at: sourceURL, to: destinationURL)
} catch {
    print("Extraction of ZIP archive failed with error: \(error)")
    exit(1)
}
