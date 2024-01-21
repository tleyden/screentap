//// https://stackoverflow.com/questions/71491312/creating-video-from-images-with-avassetwriter-some-frames-are-black
//
//
//class MovieMaker {
//    let outputSettings: MovieOutputSettings
//    let assetWriter: AVAssetWriter
//    let input: AVAssetWriterInput
//    let pixelBufferAdaptor: AVAssetWriterInputPixelBufferAdaptor!
//    var currentFrame = 0
//    let timescale: Int32 = 600
//
//    // I added this to keep track of which image is being added
//    // this helps when calculating the time
//    var imageIndex = 0
//
//    init(outputSettings: MovieOutputSettings) {
//        self.outputSettings = outputSettings
//
//        do {
//            self.assetWriter = try AVAssetWriter(outputURL: outputSettings.outputURL, fileType: .mp4)
//
//            let outputSettings = [AVVideoCodecKey : AVVideoCodecType.h264,
//                                  AVVideoWidthKey : NSNumber(floatLiteral: Double(outputSettings.size.width)),
//                                  AVVideoHeightKey : NSNumber(floatLiteral: Double(outputSettings.size.height))] as [String : Any]
//
//            self.input = AVAssetWriterInput(mediaType: .video, outputSettings: outputSettings)
//            self.assetWriter.add(input)
//            print(assetWriter.canApply(outputSettings: AVOutputSettingsAssistant(preset: .hevc3840x2160WithAlpha)?.videoSettings!, forMediaType: .video))
//        } catch {
//            print("error: \(error)")
//            fatalError()
//        }
//
//        // Set the required attributes for your pixel buffer adaptor
//        let pixelBufferAttributes = [
//            kCVPixelBufferPixelFormatTypeKey as String: NSNumber(value: kCVPixelFormatType_32ARGB),
//            kCVPixelBufferWidthKey as String: NSNumber(value: Float(outputSettings.size.width)),
//            kCVPixelBufferHeightKey as String: NSNumber(value: Float(outputSettings.size.height))
//        ]
//
//        pixelBufferAdaptor = AVAssetWriterInputPixelBufferAdaptor(assetWriterInput: input,
//                                                                  sourcePixelBufferAttributes: pixelBufferAttributes)
//    }
//
//    // I updated this function of yours to buffer
//    func pixelBufferFrom(_ image: UIImage,
//                         pixelBufferPool: CVPixelBufferPool,
//                         size: CGSize) -> CVPixelBuffer {
//
//        var pixelBufferOut: CVPixelBuffer?
//
//        let status = CVPixelBufferPoolCreatePixelBuffer(kCFAllocatorDefault,
//                                                        pixelBufferPool,
//                                                        &pixelBufferOut)
//
//        if status != kCVReturnSuccess {
//            fatalError("CVPixelBufferPoolCreatePixelBuffer() failed")
//        }
//
//        let pixelBuffer = pixelBufferOut!
//
//        CVPixelBufferLockBaseAddress(pixelBuffer,
//                                     CVPixelBufferLockFlags(rawValue: 0))
//
//        let data = CVPixelBufferGetBaseAddress(pixelBuffer)
//        let rgbColorSpace = CGColorSpaceCreateDeviceRGB()
//
//        let context = CGContext(data: data,
//                                width: Int(size.width),
//                                height: Int(size.height),
//                                bitsPerComponent: 8,
//                                bytesPerRow: CVPixelBufferGetBytesPerRow(pixelBuffer),
//                                space: rgbColorSpace,
//                                bitmapInfo: CGImageAlphaInfo.premultipliedFirst.rawValue)
//
//        context?.clear(CGRect(x:0,
//                              y: 0,
//                              width: size.width,
//                              height: size.height))
//
//        let horizontalRatio = size.width / image.size.width
//        let verticalRatio = size.height / image.size.height
//
//        // ScaleAspectFit
//        let aspectRatio = min(horizontalRatio,
//                              verticalRatio)
//
//        let newSize = CGSize(width: image.size.width * aspectRatio,
//                             height: image.size.height * aspectRatio)
//
//        let x = newSize.width < size.width ? (size.width - newSize.width) / 2 : 0
//        let y = newSize.height < size.height ? (size.height - newSize.height) / 2 : 0
//
//        context?.draw(image.cgImage!,
//                      in: CGRect(x:x,
//                                 y: y,
//                                 width: newSize.width,
//                                 height: newSize.height))
//
//        CVPixelBufferUnlockBaseAddress(pixelBuffer,
//                                       CVPixelBufferLockFlags(rawValue: 0))
//
//        return pixelBuffer
//    }
//
//    func start() {
//        assetWriter.startWriting()
//        assetWriter.startSession(atSourceTime: .zero)
//    }
//
//    func addImage(image: UIImage) {
//        // We don't set the start time and the end time but
//        // rather the duration of each frame
//        let frameDuration = CMTimeMake(value: Int64(timescale / Int32(outputSettings.fps)),
//                                       timescale: timescale)
//
//        imageIndex += 1
//
//        // Keep adding the image for the required number of frames
//        while currentFrame < imageIndex * (outputSettings.fps * Int(outputSettings.lengthPerImage))
//        {
//            // Convert the frame duration into the presentation time
//            let presentationTime = CMTimeMultiply(frameDuration,
//                                                  multiplier: Int32(currentFrame))
//
//            let pixelBuffer = pixelBufferFrom(image,
//                                              pixelBufferPool: pixelBufferAdaptor.pixelBufferPool!,
//                                              size: outputSettings.size)
//
//            print("is ready", pixelBufferAdaptor.assetWriterInput.isReadyForMoreMediaData)
//            pixelBufferAdaptor.append(pixelBuffer,
//                                      withPresentationTime: presentationTime)
//
//            currentFrame += 1
//        }
//    }
//
//    func finish(completion: @escaping (URL?) -> ()) {
//        input.markAsFinished()
//
//        // Reset the index
//        imageIndex = 0
//
//        assetWriter.finishWriting {
//            print(self.outputSettings.outputURL)
//            completion(self.outputSettings.outputURL)
//        }
//    }
//}
