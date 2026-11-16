//
//  stage-ocr — Apple Vision OCR for Stage stills.
//
//  Reads each image with VNRecognizeTextRequest and prints one JSON object
//  per image on stdout: pixel boxes from the top-left, confidence 0-100.
//  Language correction is off: the stills carry names and codes that a
//  spellchecker would "fix". Errors go to stderr, never stdout.
//

import Foundation
import ImageIO
import Vision

struct Word: Codable {
    let text: String
    let conf: Double
    let x: Int
    let y: Int
    let w: Int
    let h: Int
}

struct ImageResult: Codable {
    let path: String
    let width: Int
    let height: Int
    let words: [Word]
}

struct Options {
    var paths: [String] = []
    var minConf: Double = 0
    var level: VNRequestTextRecognitionLevel = .accurate
}

func fail(_ message: String, _ code: Int32 = 1) -> Never {
    FileHandle.standardError.write(Data("stage-ocr: \(message)\n".utf8))
    exit(code)
}

func parseArguments(_ arguments: [String]) -> Options {
    var options = Options()
    var index = 0
    while index < arguments.count {
        switch arguments[index] {
        case "--min-conf":
            index += 1
            guard index < arguments.count, let value = Double(arguments[index]) else {
                fail("--min-conf needs a number", 2)
            }
            options.minConf = value
        case "--level":
            index += 1
            guard index < arguments.count else { fail("--level needs fast or accurate", 2) }
            switch arguments[index] {
            case "fast": options.level = .fast
            case "accurate": options.level = .accurate
            default: fail("--level must be fast or accurate", 2)
            }
        default:
            options.paths.append(arguments[index])
        }
        index += 1
    }
    if options.paths.isEmpty {
        fail("usage: stage-ocr <image-path>... [--min-conf N] [--level fast|accurate]", 2)
    }
    return options
}

func loadImage(_ path: String) -> CGImage {
    let url = URL(fileURLWithPath: path)
    guard let source = CGImageSourceCreateWithURL(url as CFURL, nil),
          let image = CGImageSourceCreateImageAtIndex(source, 0, nil) else {
        fail("cannot read image: \(path)")
    }
    return image
}

func words(in image: CGImage, level: VNRequestTextRecognitionLevel, minConf: Double) -> [Word] {
    let request = VNRecognizeTextRequest()
    request.recognitionLevel = level
    request.usesLanguageCorrection = false
    let handler = VNImageRequestHandler(cgImage: image, options: [:])
    do {
        try handler.perform([request])
    } catch {
        fail("Vision recognition failed: \(error.localizedDescription)")
    }
    let width = CGFloat(image.width)
    let height = CGFloat(image.height)
    var words: [Word] = []
    for observation in request.results ?? [] {
        guard let candidate = observation.topCandidates(1).first else { continue }
        let line = candidate.string
        var index = line.startIndex
        while index < line.endIndex {
            while index < line.endIndex && line[index].isWhitespace {
                index = line.index(after: index)
            }
            guard index < line.endIndex else { break }
            let start = index
            while index < line.endIndex && !line[index].isWhitespace {
                index = line.index(after: index)
            }
            let range = start..<index
            guard let box = try? candidate.boundingBox(for: range) else { continue }
            let conf = Double(candidate.confidence) * 100
            guard conf >= minConf else { continue }
            // Vision boxes are normalized with a bottom-left origin.
            let rect = box.boundingBox
            words.append(Word(
                text: String(line[range]),
                conf: (conf * 10).rounded() / 10,
                x: Int((rect.origin.x * width).rounded()),
                y: Int(((1 - rect.origin.y - rect.size.height) * height).rounded()),
                w: Int((rect.size.width * width).rounded()),
                h: Int((rect.size.height * height).rounded())
            ))
        }
    }
    return words
}

let options = parseArguments(Array(CommandLine.arguments.dropFirst()))
let encoder = JSONEncoder()
encoder.outputFormatting = [.sortedKeys, .withoutEscapingSlashes]
for path in options.paths {
    let image = loadImage(path)
    let result = ImageResult(
        path: path,
        width: image.width,
        height: image.height,
        words: words(in: image, level: options.level, minConf: options.minConf)
    )
    guard let data = try? encoder.encode(result) else {
        fail("could not encode result for \(path)")
    }
    print(String(decoding: data, as: UTF8.self))
}
