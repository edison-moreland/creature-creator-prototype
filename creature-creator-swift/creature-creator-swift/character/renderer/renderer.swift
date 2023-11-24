//
//  renderer.swift
//  creature-creator-swift
//
//  Created by Edison Moreland on 11/24/23.
//

import Foundation
import SwiftUI
import MetalKit

struct RendererView: NSViewRepresentable {
    class Coordinator: NSObject, MTKViewDelegate {
        var device: MTLDevice
        
        private var camera: Camera
    
        private var commandQueue: MTLCommandQueue
        private var depthStencil: MTLDepthStencilState
    
        private var linePipeline: LinePipeline
    
        override init() {
            device = MTLCreateSystemDefaultDevice()!
        
            commandQueue = device.makeCommandQueue()!
        
            let depthDescriptor = MTLDepthStencilDescriptor()
            depthDescriptor.isDepthWriteEnabled = true
            depthDescriptor.depthCompareFunction = .lessEqual
            depthStencil = device.makeDepthStencilState(descriptor: depthDescriptor)!

            linePipeline = LinePipeline(device)
            
            camera = Camera(
                eye: simd_float3(-100, 0, 0),
                target: simd_float3(0, 0, 0),
                fov: 90,
                aspectRatio: 1
            )
        
            super.init()
        }
        
        func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
            self.camera.aspectRatioUpdated(aspectRatio: Float(size.width / size.height))
        }
        
        func encodeCommands(_ view: MTKView, _ uniforms: inout Uniforms, _ lines: [LineSegment]) {
            let buffer = commandQueue.makeCommandBuffer()!
            let descriptor = view.currentRenderPassDescriptor!
            let encoder = buffer.makeRenderCommandEncoder(descriptor: descriptor)!
            
            encoder.setDepthStencilState(depthStencil)
            encoder.setVertexBytes(&uniforms, length: MemoryLayout<Uniforms>.size, index: 2)
                
            linePipeline.draw(encoder: encoder, lineSegments: lines)

            encoder.endEncoding()
            
            buffer.present(view.currentDrawable!)
            buffer.commit()
        }

        func draw(in view: MTKView) {
            var uniforms = camera.uniforms()
            
            let lines = [
            	lineSegment(
                    from: simd_float3(0, 0, -5),
                    to: simd_float3(0, 0, 5),
                    color: simd_float3(),
                    shape: 0,
                    thickness: 1.0,
                    dash_size: 0.0,
                    dash_offset: 0.0
                )
            ]
            
            encodeCommands(view, &uniforms, lines)
        }
    }
    
    func makeCoordinator() -> Coordinator { Coordinator() }

    func makeNSView(context: Context) -> MTKView {
        let view = MTKView()

        view.delegate = context.coordinator
        view.device = context.coordinator.device
        view.colorPixelFormat = .rgba8Unorm
        view.clearColor = MTLClearColor(red: 1, green: 1, blue: 1, alpha: 1)
        view.depthStencilPixelFormat = .depth32Float
        view.depthStencilAttachmentTextureUsage = .renderTarget
        view.depthStencilStorageMode = .memoryless
        view.clearDepth = 1.0
        view.drawableSize = view.frame.size
        view.preferredFramesPerSecond = 60
        view.enableSetNeedsDisplay = false
        view.needsDisplay = true
        view.isPaused = false

        return view
    }

    func updateNSView(_ nsView: MTKView, context: Context) { }
}
