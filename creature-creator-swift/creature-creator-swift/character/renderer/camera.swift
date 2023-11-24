//
//  camera.swift
//  creature-creator-swift
//
//  Created by Edison Moreland on 11/24/23.
//

import Foundation

func inverseLookatMatrix(
    eye: simd_float3,
    at: simd_float3,
    up: simd_float3
) -> simd_float4x4 {
    let z = normalize(at - eye)
    let x = normalize(cross(up, z))
    let y = cross(z, x)
    let t = simd_float3(
        x: -dot(x, eye),
        y: -dot(y, eye),
        z: -dot(z, eye)
    )
    
    return simd_float4x4(rows: [
        simd_float4(x.x, y.x, z.x, 0),
        simd_float4(x.y, y.y, z.y, 0),
        simd_float4(x.z, y.z, z.z, 0),
        simd_float4(t.x, t.x, t.x, 1)
    ])
}

func projectionMatrix(
    viewAngle: Float,
    aspectRatio: Float,
    nearPlane: Float,
    farPlane: Float
) -> simd_float4x4 {
    let va_tan = 1.0 / tanf(viewAngle * 0.5)
    let ys = va_tan
    let xs = ys / aspectRatio
    let zs = farPlane / (farPlane - nearPlane)
        
    return simd_float4x4(rows: [
        simd_float4(xs, 0,  0, 0),
        simd_float4(0, ys,  0, 0),
        simd_float4(0,  0, zs, 0),
        simd_float4(0,  0, -nearPlane * zs, 1)
    ])
}

struct Camera {
    private var projection: simd_float4x4
    private var view: simd_float4x4
    
    private var eye: simd_float3
    private var target: simd_float3
    private var aspectRatio: Float
    private var fov: Float
    
    init(
        eye: simd_float3,
        target: simd_float3,
        fov: Float,
        aspectRatio: Float
    ) {
        self.eye = eye
        self.target = target
        self.fov = fov
        self.aspectRatio = aspectRatio
        
        self.projection = projectionMatrix(
            viewAngle: fov,
            aspectRatio: aspectRatio,
            nearPlane: 0.001,
            farPlane: 10000
        )
        
        // let direction = normalize(direction)
        // let right = normalize(cross(direction, up))
        // let up = cross(right, direction)
        self.view = inverseLookatMatrix(
            eye: eye,
            at: target,
            up: simd_float3(0, 1, 0)
        )
    }
    
    mutating func aspectRatioUpdated(aspectRatio: Float) {
        self.aspectRatio = aspectRatio
        self.projection = projectionMatrix(
            viewAngle: self.fov,
            aspectRatio: self.aspectRatio,
            nearPlane: 0.001,
            farPlane: 10000
        )
    }
    
    func uniforms() -> Uniforms {
        return Uniforms(
            camera: projection * view,
            cameraPosition: self.eye
        )
    }
}
