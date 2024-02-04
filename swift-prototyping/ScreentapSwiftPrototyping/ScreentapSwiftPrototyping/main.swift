
import Foundation
import Foundation
import Vision
import ScreenCaptureKit
import CoreGraphics
import AVFoundation
import ImageIO


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
            
            let documentsDirectory = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
            let fileName = "image_\(frameNumber).png" // Customize the file name as needed
            let fileURL = documentsDirectory.appendingPathComponent(fileName)

            // Create the destination using the file URL
            if let destination = CGImageDestinationCreateWithURL(fileURL as CFURL, kUTTypePNG, 1, nil) {
                // Add the CGImage to the destination
                CGImageDestinationAddImage(destination, image, nil)
                
                // Finalize the destination to actually write the image to disk
                if !CGImageDestinationFinalize(destination) {
                    print("Failed to write image to \(fileURL)")
                } else {
                    print("Image successfully written to \(fileURL)")
                }
            }
            
        }
        frameNumber += 1

        print("Captured image \(frameNumber)")
                
        // TODO: make this configurable.  Sync the frameDuration = CMTime(..) with it to match
        Thread.sleep(forTimeInterval: 1.0)

        if imageBatch.count >= 5 {
            
            let targetFilename = "/tmp/screencapture_\(dateString)_\(batchNumber).mp4"
         
            let documentsDirectory = FileManager.default.urls(for: .documentDirectory, in: .userDomainMask).first!
            
            // let documentsDirectoryStr = "/tmp"
            let documentsDirectoryStr = documentsDirectory.path
            
            swiftWriteImagesInDirToMp4(documentsDirectoryStr, targetFilename: targetFilename)
            
            let targetFilenameUrl = URL(fileURLWithPath: targetFilename)
            
            if let fps = getVideoFPS(from: targetFilenameUrl) {
                print("FPS: \(fps)")
                // iterateOverMP4Frames(url: targetFilenameUrl, fps: fps)
                
                let extractFrameID = 4
                if let extractedCGImage = getCGImageFromMP4Frame(url: targetFilenameUrl, fps: fps, frameID: ) {
                    let path = "/tmp/cgimage_extracted.png"
                    writeCGImage(extractedCGImage, toPath: path)
                    print("Wrote extracted cgimage frame to \(path)")
                } else {
                    print("Failed to extract CGImage.")
                }
                
                
            } else {
                print("Failed to retrieve FPS")
            }
            
            
                        
            imageBatch.removeAll() // Clear the batch after writing
            
            batchNumber += 1
        }
        
    }
    
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

func swiftWriteImagesInDirToMp4(_ directoryPath: String, targetFilename: String) {
    

    let directoryURL = URL(fileURLWithPath: directoryPath)

    let images = fetchSortedPngImages(from: directoryURL)
    print("Writing images to mp4: \(images)")
    
    swiftWriteImagesToMp4(images, targetFilename: targetFilename)
    
    
}

//func swiftWriteImagesInDirToMp4(_ directory: URL, targetFilename: String) {
//
//    let images = fetchSortedPngImages(from: directory)
//
//    swiftWriteImagesToMp4(images, targetFilename: targetFilename)
//
//}


// Define swiftWriteImagesToMp4
func swiftWriteImagesToMp4(_ images: [CGImage], targetFilename: String, blockUntilFinished: Bool = true) {

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

    if blockUntilFinished {
        print("Call semaphore.wait()")
        semaphore.wait() // Wait for the signal
        print("Semaphore.wait() returned")
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
        // This means images are dropped
        // TODO: handle this better
        print("Cannot write to pixelBufferAdaptor because isReadyForMoreMediaData is false.")
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

func getVideoFPS(from url: URL) -> Float? {
    let asset = AVAsset(url: url)
    let tracks = asset.tracks(withMediaType: .video)
    
    guard let track = tracks.first else {
        return nil
    }
    
    return track.nominalFrameRate
}

func iterateOverMP4Frames(url: URL, fps: Float) {
    
    let asset = AVAsset(url: url)

    // Create an AVAssetImageGenerator
    let imageGenerator = AVAssetImageGenerator(asset: asset)
    imageGenerator.appliesPreferredTrackTransform = true
    imageGenerator.requestedTimeToleranceBefore = .zero //Optional
    imageGenerator.requestedTimeToleranceAfter = .zero //Optional

    // Get the total number of frames
    let duration = asset.duration
    let durationSeconds = CMTimeGetSeconds(duration)
    let totalFrames = Int(durationSeconds * Double(fps))

    // Iterate over each frame
    for frameCount in 0..<totalFrames {
        let time = CMTimeMake(value: Int64(frameCount), timescale: Int32(fps))
        do {
            let cgImage = try imageGenerator.copyCGImage(at: time, actualTime: nil)
            // Process the frame (e.g., convert to PNG, save to file, etc.)
            let path = "/tmp/cgimage_\(frameCount).png"
            writeCGImage(cgImage, toPath: path)
            
        } catch {
            print("Error generating image at frame \(frameCount): \(error)")
        }
    }
    
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

main()
