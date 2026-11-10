#include <metal_stdlib>
using namespace metal;

kernel void vector_add(const device uint *a [[buffer(0)]],
                       const device uint *b [[buffer(1)]],
                       device uint *output [[buffer(2)]],
                       uint index [[thread_position_in_grid]]) {
    output[index] = a[index] + b[index];
}
