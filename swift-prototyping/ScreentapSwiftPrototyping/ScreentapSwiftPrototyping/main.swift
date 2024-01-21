
import Foundation
import Foundation
import Vision
import ScreenCaptureKit
import CoreGraphics
import AVFoundation


func main() {
    
    var imageBatch: [CGImage] = []

    var frameNumber: Int32 = 0
    var batchNumber: Int32 = 0

    // Get a datestring to create unique mp4 filenames
    let dateFormatter = DateFormatter()
    dateFormatter.dateFormat = "yyyyMMdd_HHmmss" // Example format: '20230121_115959'
    let dateString = dateFormatter.string(from: Date())

    while true {
        
        if let image = swiftCaptureImage(frameNumber: frameNumber) {
            imageBatch.append(image)
        }
        frameNumber += 1

        print("Captured image \(frameNumber)")
                
        // TODO: make this configurable.  Sync the frameDuration = CMTime(..) with it to match
        Thread.sleep(forTimeInterval: 1.0)

        if imageBatch.count >= 5 {
            
            let targetFilename = "/tmp/screencapture_\(dateString)_\(batchNumber).mp4"
            
            swiftWriteImagesToMp4(imageBatch, targetFilename: targetFilename)
                        
            imageBatch.removeAll() // Clear the batch after writing
            
            batchNumber += 1
        }
        
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

// Define swiftWriteImagesToMp4
func swiftWriteImagesToMp4(_ images: [CGImage], targetFilename: String) {

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

    videoWriter.finishWriting() {
        // TODO: invoke callback fn that is passed in
        print("Finished writing video to \(targetFilename) with \(images.count) images")
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

    pixelBufferAdaptor.append(pixelBuffer, withPresentationTime: presentationTime)

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

main()
