// From https://github.com/stack/advent_of_code_2021/blob/main/Utilities/Animator.swift

import AVFoundation
import Foundation

public class Animator: ObservableObject {

    public typealias DrawCallback = (CGContext) -> ()

    @Published public var latestPixelBuffer: CVPixelBuffer? = nil

    var width: Int = 400
    var height: Int = 400

    var frameTime: Double = 1.0 {
        didSet { frameRate = CMTime(seconds: frameTime, preferredTimescale: CMTimeScale(NSEC_PER_SEC)) }
    }

    var url: URL? = nil
    var backPressure: Int = 32

    private var frameRate: CMTime = CMTime(seconds: 1.0, preferredTimescale: CMTimeScale(NSEC_PER_SEC))

    let writerQueue: DispatchQueue
    let writerCondition: NSCondition
    var writerSemaphore: DispatchSemaphore

    var writer: AVAssetWriter!
    var writerInput: AVAssetWriterInput!
    var writerAdaptor: AVAssetWriterInputPixelBufferAdaptor!

    var writerObservation: NSKeyValueObservation!

    var currentFrameTime = CMTime(seconds: 0.0, preferredTimescale: CMTimeScale(NSEC_PER_SEC))

    /**
     Initialize a new Animator with the given dimensions and frame rate.
     */
    public init() {
        writerQueue = DispatchQueue(label: "us.gerstacker.adventofcode.animator")
        writerCondition = NSCondition()
        writerSemaphore = DispatchSemaphore(value: backPressure)
    }

    /**
     Finalize the video file.

     All submitted video frames will be drawn, encoded, and stored in the video file.

     Adding new frames after calling complete is unsupported.

     This call will block until the file writing has completed.
     */
    public func complete() {
        writerQueue.sync {
            writerInput.markAsFinished()
        }

        let condition = NSCondition()
        var complete = false

        writer.finishWriting {
            if self.writer.status == .failed {
                let message = self.writer.error?.localizedDescription ?? "UNKNOWN"
                print("Failed to finish writing: \(message)")
            }

            condition.lock()
            complete = true
            condition.signal()
            condition.unlock()
        }

        condition.lock()

        while !complete {
            condition.wait()
        }

        condition.unlock()

        writerObservation.invalidate()
    }

    /**
     Retreive a new `CGContext` to draw in.

     Once the callback is complete, but provided context is submitted for encoding and writing.

     - Parameter callback: The function that provides the `CGContext` to draw and submit a frame.
     */
    public func draw(callback: DrawCallback) {
        guard let pool = writerAdaptor.pixelBufferPool else {
            fatalError("No pixel buffer pool to pull from")
        }

        var nextPixelBuffer: CVPixelBuffer? = nil
        CVPixelBufferPoolCreatePixelBuffer(kCFAllocatorDefault, pool, &nextPixelBuffer)

        guard let pixelBuffer = nextPixelBuffer else {
            fatalError("Failed to get next pixel buffer for drawing")
        }

        CVPixelBufferLockBaseAddress(pixelBuffer, [])
        let baseAddress = CVPixelBufferGetBaseAddress(pixelBuffer)
        let width = CVPixelBufferGetWidth(pixelBuffer)
        let height = CVPixelBufferGetHeight(pixelBuffer)
        let stride = CVPixelBufferGetBytesPerRow(pixelBuffer)

        let colorSpace = CGColorSpaceCreateDeviceRGB()
        guard let context = CGContext(data: baseAddress, width: width, height: height, bitsPerComponent: 8, bytesPerRow: stride, space: colorSpace, bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue + CGBitmapInfo.byteOrder32Little.rawValue) else {
            fatalError("Failed to create context")
        }

        context.translateBy(x: 0.0, y: CGFloat(height))
        context.scaleBy(x: 1.0, y: -1.0)

        callback(context)

        CVPixelBufferUnlockBaseAddress(pixelBuffer, [])
        
        DispatchQueue.main.async {
            self.latestPixelBuffer = pixelBuffer
        }
        
        submit(pixelBuffer: pixelBuffer)
    }
    
    public func repeatLastFrame() {
        guard let pixelBuffer = self.latestPixelBuffer else { return }
        submit(pixelBuffer: pixelBuffer)
    }
    
    private func submit(pixelBuffer: CVPixelBuffer) {
        writerSemaphore.wait()

        writerQueue.async {
            self.writerCondition.lock()

            while !self.writerInput.isReadyForMoreMediaData {
                self.writerCondition.wait()
            }

            self.writerCondition.unlock()

            self.writerAdaptor.append(pixelBuffer, withPresentationTime: self.currentFrameTime)
            self.currentFrameTime = CMTimeAdd(self.currentFrameTime, self.frameRate)

            self.writerSemaphore.signal()
        }
    }

    public func start() {
        precondition(width % 2 == 0)
        precondition(height % 2 == 0)
        precondition(url != nil)

        if FileManager.default.fileExists(atPath: url!.path) {
            try! FileManager.default.removeItem(at: url!)
        }

        writer = try! AVAssetWriter(url: url!, fileType: .mov)

        let videoSettings: [String:Any] = [
            AVVideoCodecKey: AVVideoCodecType.hevc,
            AVVideoWidthKey: NSNumber(value: width),
            AVVideoHeightKey: NSNumber(value:height),
        ]

        writerInput = AVAssetWriterInput(mediaType: .video, outputSettings: videoSettings)

        let sourceAttributes: [String:Any] = [
            String(kCVPixelBufferPixelFormatTypeKey): NSNumber(value: kCVPixelFormatType_32BGRA),
            String(kCVPixelBufferMetalCompatibilityKey) : NSNumber(value: true)
        ]

        writerAdaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: writerInput, sourcePixelBufferAttributes: sourceAttributes)

        if writer.canAdd(writerInput) {
            writer.add(writerInput)
        } else {
            fatalError("Could not add writer input")
        }

        guard writer.startWriting() else {
            let message = writer.error?.localizedDescription ?? "UNKNOWN"
            fatalError("Could not start writing: \(message)")
        }

        writer.startSession(atSourceTime: currentFrameTime)

        writerSemaphore = DispatchSemaphore(value: backPressure)

        writerObservation = writerInput.observe(\.isReadyForMoreMediaData, options: .new, changeHandler: { (_, change) in
            guard let isReady = change.newValue else {
                return
            }

            if isReady {
                self.writerCondition.lock()
                self.writerCondition.signal()
                self.writerCondition.unlock()
            }
        })
    }
}
