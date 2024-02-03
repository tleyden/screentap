import Foundation
import SwiftRs
import Vision
import ScreenCaptureKit
import CoreGraphics
import AVFoundation


/**
 * Write all of the PNG images in a directory into an mp4 file given by 
 * targetFilename
 */
@_cdecl("write_images_in_dir_to_mp4_swift")
@available(macOS 10.15, *)
func swiftWriteImagesInDirToMp4(_ directoryPath: SRString, targetFilename: SRString) {  
    
    let directoryURL = URL(fileURLWithPath: directoryPath.toString())
    print("Directory URL: \(directoryURL)")

    let images = fetchSortedPngImages(from: directoryURL)
    print("Number of images: \(images.count)")

    print("swiftWriteImagesToMp4 to: \(targetFilename.toString())")

    swiftWriteImagesToMp4(images, targetFilename: targetFilename.toString())

    print("swiftWriteImagesToMp4 finished")
    
}

/**
 * Capture a few screenshots and write them to an mp4 file
 */
@_cdecl("cap_screenshot_to_mp4_swift")
@available(macOS 10.15, *)
public func cap_screenshot_to_mp4(screenshot: SRData) -> SRString? {

    do {

        var imageBatch: [CGImage] = []

        var frameNumber: Int32 = 0
        var batchNumber: Int32 = 0

        // Get a datestring to create unique mp4 filenames
        let dateFormatter = DateFormatter()
        dateFormatter.dateFormat = "yyyyMMdd_HHmmss" // Example format: '20230121_115959'
        let dateString = dateFormatter.string(from: Date())

        // Convert the SwiftRs array of SRData to an array of CGImages
        // for screenshot in screenshots {
        //     if let cgImage = byteArrayToCGImage(byteArray: screenshot.toArray()) {
        //         imageBatch.append(cgImage)
        //     }
        // }
        
        if let cgImage = byteArrayToCGImage(byteArray: screenshot.toArray()) {
            imageBatch.append(cgImage)
        }

        let targetFilename = "/tmp/screencapture_\(dateString)_\(batchNumber).mp4"
                
        swiftWriteImagesToMp4(imageBatch, targetFilename: targetFilename)


        // while true {
            
        //     if let image = swiftCaptureImage(frameNumber: frameNumber) {
        //         imageBatch.append(image)
        //     }
        //     frameNumber += 1

        //     print("Captured image \(frameNumber)")
                    
        //     // TODO: make this configurable.  Sync the frameDuration = CMTime(..) with it to match
        //     Thread.sleep(forTimeInterval: 1.0)

        //     if imageBatch.count >= 5 {
                
        //         let targetFilename = "/tmp/screencapture_\(dateString)_\(batchNumber).mp4"
                
        //         swiftWriteImagesToMp4(imageBatch, targetFilename: targetFilename)
                            
        //         imageBatch.removeAll() // Clear the batch after writing
                
        //         batchNumber += 1
        //     }
            
        // }


        return SRString("Video written to file")

    } catch {
        // This block is executed if an error is thrown in the do block
        // You can handle the error here
        // For example, you can return a string indicating the error
        return SRString("An error occurred: \(error)")
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




// Define swiftWriteImagesToMp4
func swiftWriteImagesToMp4(_ images: [CGImage], targetFilename: String) {

    print("swiftWriteImagesToMp4 running...")


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

    let videoSettings: [String: Any] = [
        AVVideoCodecKey: AVVideoCodecType.h264,
        AVVideoWidthKey: imageWidth,
        AVVideoHeightKey: imageHeight
    ]

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

    print("Call videoWriter.finishWriting()")
    videoWriter.finishWriting() {
        // TODO: invoke callback fn that is passed in
        print("Finished writing video to \(targetFilename) with \(images.count) images")
        semaphore.signal() // Signal the semaphore to end waiting
    }

    print("Call semaphore.wait()")
    semaphore.wait() // Wait for the signal
    print("Semaphore.wait() returned")

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
