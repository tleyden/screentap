import Foundation
import SwiftRs
import Vision

@_cdecl("perform_ocr_swift")
@available(macOS 10.15, *)
public func perform_ocr(path: SRString) -> SRString? {
    let fileUrl = URL(fileURLWithPath: path.toString())
    if fileUrl.pathExtension != "png" {
        return nil
    }
    guard let imageSource = CGImageSourceCreateWithURL(fileUrl as CFURL, nil) else {
        print("Can't open '\(fileUrl)'!")
        return nil
    }
    let imageOptions: [CFString: Any] = [
        kCGImageSourceShouldCache: false,
        kCGImageSourceShouldAllowFloat: false,
    ]

    guard let image = CGImageSourceCreateImageAtIndex(imageSource, 0, imageOptions as CFDictionary) else {
        print("Cannot create CGImage for: " + fileUrl.path)
        return nil
    }
    let request = VNRecognizeTextRequest(completionHandler: { _, error in
        if let error = error {
            print("Error: \(error)")
            return
        }
    })
    request.recognitionLevel = .accurate
    request.usesLanguageCorrection = true
    let handler = VNImageRequestHandler(cgImage: image, options: [:])
    do {
        try handler.perform([request])
        guard let observations = request.results else {
            fatalError("Error translating image")
        }
        let recognizedText = observations.compactMap { observation in
            observation.topCandidates(1).first?.string
        }.joined(separator: "\n")
        return SRString(recognizedText)
    } catch {
        print("Error: \(error)")
        return nil
    }
}
