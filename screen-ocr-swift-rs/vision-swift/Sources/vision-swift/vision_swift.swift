import Foundation
import SwiftRs
import Vision
import ScreenCaptureKit
import CoreGraphics
import AVFoundation
import Cocoa

/**
 * Write all of the PNG images in a directory into an mp4 file given by 
 * targetFilename
 */
@_cdecl("write_images_in_dir_to_mp4_swift")
@available(macOS 10.15, *)
func swiftWriteImagesInDirToMp4(_ directoryPath: SRString, targetFilename: SRString, useBitRateKey: Bool = false) {  
    
    let directoryURL = URL(fileURLWithPath: directoryPath.toString())

    let images = fetchSortedPngImages(from: directoryURL)

    if images.isEmpty {
        print("No PNG images found in \(directoryURL.path)")
        return
    }

    swiftWriteImagesToMp4(
        images, 
        targetFilename: targetFilename.toString(),
        blockUntilFinished: true,
        useBitRateKey: useBitRateKey
    )
    
}

@_cdecl("extract_frame_from_mp4_swift")
@available(macOS 10.15, *)
public func extract_frame_from_mp4(mp4_path: SRString, frame_id: Int) -> SRData? {

    let mp4Url = URL(fileURLWithPath: mp4_path.toString())

    if let fps = getVideoFPS(from: mp4Url) {
        
        if let extractedCGImage = getCGImageFromMP4Frame(url: mp4Url, fps: fps, frameID: frame_id) {
            
            if let byteArray = convertCGImageToByteArray(image: extractedCGImage) {
                return SRData(byteArray)
            } else {
                print("Failed to convert CGImage to byte array")
            }

        } else {
            print("Failed to extract CGImage.")
        }
        
        
    } else {
        print("Failed to retrieve FPS")
    }

    return nil
}

/**
 * NOTE: no longer used because it was returning stale values, and using KVO observing
 * or NSWorkspace.DidActivateApplicationNotification appears to be difficult to do
 * with the swift-rs bridge.
 */
@_cdecl("get_frontmost_app_swift")
@available(macOS 10.15, *)
public func get_frontmost_app() -> SRString {
    if let appname = NSWorkspace.shared.frontmostApplication {
        if let localized_name = appname.localizedName {
            return SRString(localized_name)
        } else {
            return SRString("")
        }
    } else {
        return SRString("")
    }
}

@_cdecl("resize_image_swift")
@available(macOS 10.15, *)
public func resize_image(image: SRData) -> SRData? {

    // Convert the byte array to CGImage
    let byteArray = image.toArray()

    if let cgImage = byteArrayToCGImage(byteArray: byteArray) {

        // Resize the image
        if let resizedCGImage = resizedImage(image: cgImage, scale: 0.5) {

            // Convert the resized CGImage to byte array
            if let resizedByteArray = convertCGImageToByteArray(image: resizedCGImage) {
                return SRData(resizedByteArray)
            } else {
                print("Failed to convert resized CGImage to byte array")
            }

        } else {
            print("Failed to resize image")
        }

    } else {
        print("Failed to convert byte array to CGImage")
    }

    return nil
}

/**
 * Capture the screen and return the image as a PNG encoded byte array
 */
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

// Define swiftCaptureImage() returning an optional UIImage
func swiftCaptureImage(frameNumber: Int32) -> CGImage? {
    let displayID = CGMainDisplayID()

    // Capture the screen image
    if let image = CGDisplayCreateImage(displayID) {

        // Convert the CGImage to NSData
        let bitmapRep = NSBitmapImageRep(cgImage: image)
        
        if (!isImage32ARGB(image)) {
            print("Unexpected image format")
            return nil
        }

        return image
        
    } else {
        print("Failed to capture screen")
    }
    
    return nil
}


func fetchSortedPngImages(from directory: URL) -> [CGImage] {
    do {
        // Get the list of files in the documents directory
        let fileManager = FileManager.default
        let files = try fileManager.contentsOfDirectory(at: directory, includingPropertiesForKeys: [.creationDateKey], options: [.skipsHiddenFiles])
        
        // Filter PNG files and sort them by creation date
        let pngFiles = files.filter { $0.pathExtension == "png" }
        let sortedPngFiles = pngFiles.sorted {
            let creationDate1 = try? $0.resourceValues(forKeys: [.creationDateKey]).creationDate
            let creationDate2 = try? $1.resourceValues(forKeys: [.creationDateKey]).creationDate
            return creationDate1 ?? Date.distantPast < creationDate2 ?? Date.distantFuture
        }
        
        // Convert sorted PNG files to CGImage
        var images: [CGImage] = []
        for fileUrl in sortedPngFiles {
            if let imageSource = CGImageSourceCreateWithURL(fileUrl as CFURL, nil),
               let image = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) {
                images.append(image)
            }
        }
        
        return images
    } catch {
        print("Error fetching PNG files: \(error)")
        return []
    }
}

/**
 * Write an array of CGImages to an mp4 file given by targetFilename
 * 
 * Parameters:
 * 
 * - useBitRateKey: If true, it will use AVVideoAverageBitRateKey instaed of AVVideoQualityKey, since the latter 
 *                  is not supported on all devices.  The AVVideoQualityKey is problematic as described in
 *                  https://stackoverflow.com/questions/76811431/avfoundation-compression-property-quality-is-not-supported-for-video-codec-type/76848093#76848093
 *                  and https://forums.developer.apple.com/forums/thread/734885
 */
func swiftWriteImagesToMp4(_ images: [CGImage], targetFilename: String, blockUntilFinished: Bool = true, useBitRateKey: Bool = false) {

    let outputURL = URL(fileURLWithPath: targetFilename)

    // The targetFilename should not exist
    if FileManager.default.fileExists(atPath: outputURL.path) {
        print("File exists at \(outputURL.path), this should be called with a file that does not exist")
        return
    }
    
    guard let videoWriter = try? AVAssetWriter(outputURL: outputURL, fileType: .mp4) else {
        print("Unable to create videoWriter for \(targetFilename)")
        return
    }
    
    let imageWidth = images[0].width
    let imageHeight = images[0].height

    // See comments in method definition for explanation of useBitRateKey
    let videoSettings: [String: Any]
    if useBitRateKey {
        videoSettings = [
            AVVideoCodecKey: AVVideoCodecType.h264,
            AVVideoWidthKey: imageWidth,
            AVVideoHeightKey: imageHeight,
            AVVideoCompressionPropertiesKey: [
                AVVideoAverageBitRateKey: 1000000
            ]
        ]
    } else {
        videoSettings = [
            AVVideoCodecKey: AVVideoCodecType.h264,
            AVVideoWidthKey: imageWidth,
            AVVideoHeightKey: imageHeight,
            AVVideoCompressionPropertiesKey: [
                AVVideoQualityKey: 0.4
            ]
        ]
    }

    let videoWriterInput = AVAssetWriterInput(
        mediaType: .video,
        outputSettings: videoSettings
    )
    
    let pixelBufferAttributes: [String: Any] = [
        kCVPixelBufferPixelFormatTypeKey as String: kCVPixelFormatType_32ARGB,
        kCVPixelBufferWidthKey as String: Int(imageWidth),
        kCVPixelBufferHeightKey as String: Int(imageHeight)
    ]
    let pixelBufferAdaptor = AVAssetWriterInputPixelBufferAdaptor(
        assetWriterInput: videoWriterInput,
        sourcePixelBufferAttributes: pixelBufferAttributes
    )
    
    // Add input and start writing
    videoWriter.add(videoWriterInput)
    videoWriter.startWriting()
    videoWriter.startSession(atSourceTime: .zero)

    let frameDuration = CMTime(seconds: 1, preferredTimescale: 1)
    
    var currentPresentationTime = CMTime.zero
    
    for cgImage in images {
        
        appendPixelBuffer(
            forImage: cgImage,
            pixelBufferAdaptor: pixelBufferAdaptor,
            presentationTime: currentPresentationTime
        )
        
        // Increment the presentation time by one second for the next frame
        currentPresentationTime = CMTimeAdd(currentPresentationTime, frameDuration)
        
    }
    
    videoWriterInput.markAsFinished()

    // Need to use a semaphore to wait for the video to finish writing, otherwise when calling from
    // rust it seems to return before the video is finished writing, and variables are deallocated / GC'd
    // (this doesn't happen when running directly from swift, only when calling from rust)
    let semaphore = DispatchSemaphore(value: 0)

    videoWriter.finishWriting() {
        semaphore.signal() // Signal the semaphore to end waiting
    }

    if blockUntilFinished {
        semaphore.wait() // Wait for the signal
    }

}

private func appendPixelBuffer(
    forImage image: CGImage,
    pixelBufferAdaptor: AVAssetWriterInputPixelBufferAdaptor,
    presentationTime: CMTime) {
    
    guard let pixelBufferPool = pixelBufferAdaptor.pixelBufferPool else {
        print("Cannot get pixelBufferPool")
        return
    }

    var pixelBufferOut: CVPixelBuffer?
    CVPixelBufferPoolCreatePixelBuffer(nil, pixelBufferPool, &pixelBufferOut)

    guard let pixelBuffer = pixelBufferOut else {
        // TODO: Return error
        print("pixelBufferOut is empty")
        return
        
    }

    CVPixelBufferLockBaseAddress(pixelBuffer, [])
    let pixelData = CVPixelBufferGetBaseAddress(pixelBuffer)

    let bitsPerComponent = image.bitsPerComponent
    let context = CGContext(
        data: pixelData,
        width: image.width,
        height: image.height,
        bitsPerComponent: bitsPerComponent,
        bytesPerRow: CVPixelBufferGetBytesPerRow(pixelBuffer),
        space: CGColorSpaceCreateDeviceRGB(),
        bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue
    )

    context?.draw(
        image,
        in: CGRect(
            x: 0,
            y: 0,
            width: image.width,
            height: image.height
        )
    )

    if pixelBufferAdaptor.assetWriterInput.isReadyForMoreMediaData {
        pixelBufferAdaptor.append(pixelBuffer, withPresentationTime: presentationTime)
    } else {
        // This means images are dropped.  I was seeing this consistently with 10x10 images,
        // but I have no idea why.  Using larger images (3000x2000) made the issue go away.
        // Then after some other changes, like adding the semaphor to block the thread until
        // the video writer finished, I can no longer reproduce the issue and it works with
        // 10x10 images.
        print("WARNING: Cannot write to pixelBufferAdaptor because isReadyForMoreMediaData is false.")
    }

    CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
}

func isImage32ARGB(_ cgImage: CGImage) -> Bool {
    
    // Check bits per component
    let bitsPerComponent = cgImage.bitsPerComponent
    if bitsPerComponent != 8 {
        return false
    }

    // Check the number of components
    guard let colorSpace = cgImage.colorSpace else { return false }
    let numberOfComponents = colorSpace.numberOfComponents
    if numberOfComponents != 3 { // RGB (not counting alpha)
        return false
    }

    // Check alpha info
    let alphaInfo = cgImage.alphaInfo
    if alphaInfo != .premultipliedFirst && alphaInfo != .first && alphaInfo != .noneSkipFirst {
        return false
    }

    // Check if color space is RGB
    if colorSpace.model != .rgb {
        return false
    }

    return true
}


func getVideoFPS(from url: URL) -> Float? {
    let asset = AVAsset(url: url)
    let tracks = asset.tracks(withMediaType: .video)
    
    guard let track = tracks.first else {
        return nil
    }
    
    return track.nominalFrameRate
}

func getCGImageFromMP4Frame(url: URL, fps: Float, frameID: Int) -> CGImage? {
    let asset = AVAsset(url: url)

    // Create an AVAssetImageGenerator
    let imageGenerator = AVAssetImageGenerator(asset: asset)
    imageGenerator.appliesPreferredTrackTransform = true
    imageGenerator.requestedTimeToleranceBefore = .zero
    imageGenerator.requestedTimeToleranceAfter = .zero

    // Calculate the CMTime for the specified frameID
    let frameTime = CMTime(value: Int64(frameID), timescale: Int32(fps))
    
    do {
        // Generate the CGImage for the specified frame
        let cgImage = try imageGenerator.copyCGImage(at: frameTime, actualTime: nil)
        // Here you can process the CGImage (e.g., save to file)
        return cgImage
    } catch {
        print("Error generating image for frameID \(frameID): \(error)")
        return nil
    }
}

func writeCGImage(_ image: CGImage, toPath path: String) {
    let url = URL(fileURLWithPath: path)
    guard let destination = CGImageDestinationCreateWithURL(url as CFURL, kUTTypePNG, 1, nil) else {
        print("Failed to create CGImageDestination for \(url)")
        return
    }
    
    CGImageDestinationAddImage(destination, image, nil)
    
    if !CGImageDestinationFinalize(destination) {
        print("Failed to write image to \(path)")
    } else {
        print("Successfully wrote image to \(path)")
    }
}

func byteArrayToCGImage(byteArray: [UInt8]) -> CGImage? {

    // Convert the byte array to Data
    let data = Data(byteArray)

    // Create a CGImageSource from the Data
    guard let imageSource = CGImageSourceCreateWithData(data as CFData, nil) else {
        print("Failed to create image source")
        return nil
    }

    // Create a CGImage from the CGImageSource
    let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil)
    return cgImage
}

func convertCGImageToByteArray(image: CGImage) -> [UInt8]? {
    // Convert the CGImage to NSData
    let bitmapRep = NSBitmapImageRep(cgImage: image)
    guard let imageData = bitmapRep.representation(using: .png, properties: [:]) else {
        print("Failed to convert image to PNG data")
        return nil
    }

    // Convert NSData to Byte Array
    let byteArray = [UInt8](imageData)
    return byteArray
}

func resizedImage(image: CGImage, scale: CGFloat) -> CGImage? {
    
    let sharedContext = CIContext(options: [.useSoftwareRenderer : false])
    
    // Convert CGImage to CIImage
    let ciImage = CIImage(cgImage: image)
    
    let filter = CIFilter(name: "CILanczosScaleTransform")
    filter?.setValue(ciImage, forKey: kCIInputImageKey)
    filter?.setValue(scale, forKey: kCIInputScaleKey)

    guard let outputCIImage = filter?.outputImage,
        let outputCGImage = sharedContext.createCGImage(outputCIImage,
                                                        from: outputCIImage.extent)
    else {
        return nil
    }

    return outputCGImage
}
