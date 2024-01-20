import Foundation
import SwiftRs
import Vision
import ScreenCaptureKit
import CoreGraphics
import AVFoundation

@_cdecl("screen_capture_swift")
@available(macOS 10.15, *)
public func screen_capture() -> SRData? {

    // Specify the display to capture (main display in this case)
    let displayID = CGMainDisplayID()

    // Capture the screen image
    if let image = CGDisplayCreateImage(displayID) {

        // Convert the CGImage to NSData
        let bitmapRep = NSBitmapImageRep(cgImage: image)
        guard let imageData = bitmapRep.representation(using: .png, properties: [:]) else {
            print("Failed to convert image to PNG data")
            return nil
        }

        // Convert NSData to Byte Array
        let byteArray = [UInt8](imageData)

        return SRData(byteArray)

        // byteArray now contains the screen capture as a Swift byte array
        // You can now use byteArray as needed
        
    } else {
        print("Failed to capture screen")
    }

    return nil

}

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

/**
 * Capture a few creenshots and write them to an mp4 file
 */
@_cdecl("cap_screenshot_to_mp4_swift")
@available(macOS 10.15, *)
public func cap_screenshot_to_mp4() -> SRString? {
    return SRString("Not implemented yet")
}