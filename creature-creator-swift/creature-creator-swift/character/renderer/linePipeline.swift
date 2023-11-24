//
//  linePipeline.swift
//  creature-creator-swift
//
//  Created by Edison Moreland on 11/24/23.
//

import Foundation
import MetalKit

class LinePipeline {
    private var pipeline: SwiftLinePipeline
    
    init(_ device: MTLDevice) {
        pipeline = SwiftLinePipeline(Unmanaged.passUnretained(device).toOpaque())
    }
    
    func draw(encoder: MTLRenderCommandEncoder, lineSegments: [LineSegment]) {
        // TODO: We need a much better way of getting primitives to rust! This is wasteful.
        let toRust = SwiftLineSegments()
        for l in lineSegments {
            toRust.push(l)
        }
        
        let encoderPtr = Unmanaged.passUnretained(encoder)
        pipeline.draw(encoderPtr.toOpaque(), toRust)
        
    }
}

func lineSegment(
    from: simd_float3,
    to: simd_float3,
    color: simd_float3,
    shape: UInt32, // 0 = rectangle, 1 = triangle
    thickness: Float,
    dash_size: Float,
    dash_offset: Float
) -> LineSegment {
    LineSegment(
        from.x, from.y, from.z,
        to.x, to.y, to.z,
        color.x, color.y, color.z,
        thickness,
        dash_size,
        shape,
        dash_offset
    )
}
