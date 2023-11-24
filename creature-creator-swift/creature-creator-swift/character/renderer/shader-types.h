//
//  shader-types.h
//  creature-creator-swift
//
//  Created by Edison Moreland on 11/20/23.
//

#ifndef shader_types_h
#define shader_types_h

#include <simd/simd.h>

struct Uniforms {
    simd_float4x4 camera;
    simd_float3 cameraPosition;
};

#endif /* shader_types_h */
