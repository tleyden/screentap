import Foundation
import SwiftRs
import Vision
import ScreenCaptureKit
import CoreGraphics
import AVFoundation

/**
 * Capture a few screenshots and write them to an mp4 file
 */
@_cdecl("cap_screenshot_to_mp4_swift")
@available(macOS 10.15, *)
public func cap_screenshot_to_mp4() -> SRString? {

    do {

        let displayID = CGMainDisplayID()

        // Setup video writer and settings
        let outputPath = "/tmp/filename.mp4"
        let outputURL = URL(fileURLWithPath: outputPath)
        guard let videoWriter = try? AVAssetWriter(outputURL: outputURL, fileType: .mp4) else {
            return SRString("Failed to create AVAssetWriter")
        }

        let videoSettings: [String: Any] = [
            AVVideoCodecKey: AVVideoCodecType.h264,
            AVVideoWidthKey: 3456,
            AVVideoHeightKey: 2234
        ]

        let videoWriterInput = AVAssetWriterInput(mediaType: .video, outputSettings: videoSettings)
        let pixelBufferAdaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: videoWriterInput, sourcePixelBufferAttributes: nil)

        // Add input and start writing
        videoWriter.add(videoWriterInput)
        videoWriter.startWriting()
        videoWriter.startSession(atSourceTime: .zero)


        var frameCount = 0

        let frameDuration = CMTimeMake(value: 1, timescale: 1)

        // Do a for loop to capture a few screenshots
        for _ in 0...10 {
            
            guard let cgImage = CGDisplayCreateImage(displayID) else {
                // Handle the nil case and return or break
                return SRString("Failed to CGDisplayCreateImage")
            }

            let presentationTime = CMTimeMake(value: Int64(frameCount), timescale: 1)
            appendPixelBuffer(forImage: cgImage, pixelBufferAdaptor: pixelBufferAdaptor, presentationTime: presentationTime)
            frameCount += 1
            print("Frame \(frameCount) added")

        }

        videoWriterInput.markAsFinished()

        videoWriter.finishWriting() {
            print("Finished writing video")
        }

        return SRString("Video written to file")

    } catch {
        // This block is executed if an error is thrown in the do block
        // You can handle the error here
        // For example, you can return a string indicating the error
        return SRString("An error occurred: \(error)")
    }


}

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

private func appendPixelBuffer(forImage image: CGImage, pixelBufferAdaptor: AVAssetWriterInputPixelBufferAdaptor, presentationTime: CMTime) {
    guard let pixelBufferPool = pixelBufferAdaptor.pixelBufferPool else { return }

    var pixelBufferOut: CVPixelBuffer?
    CVPixelBufferPoolCreatePixelBuffer(nil, pixelBufferPool, &pixelBufferOut)

    guard let pixelBuffer = pixelBufferOut else { return }

    CVPixelBufferLockBaseAddress(pixelBuffer, [])
    let pixelData = CVPixelBufferGetBaseAddress(pixelBuffer)

    let context = CGContext(data: pixelData, width: image.width, height: image.height, bitsPerComponent: 8, bytesPerRow: CVPixelBufferGetBytesPerRow(pixelBuffer), space: CGColorSpaceCreateDeviceRGB(), bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue)

    context?.draw(image, in: CGRect(x: 0, y: 0, width: image.width, height: image.height))

    pixelBufferAdaptor.append(pixelBuffer, withPresentationTime: presentationTime)

    CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
}