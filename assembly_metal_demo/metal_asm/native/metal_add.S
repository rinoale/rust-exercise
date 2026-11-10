// Apple ARM64 / Mach-O assembly. Every Metal call is made in this file.
// int asm_metal_add(const uint32_t *a, const uint32_t *b,
//                   uint32_t *output, size_t count);
// Arguments: x0=a, x1=b, x2=output, x3=count. Result: w0=status (0=success).

.section __TEXT,__text,regular,pure_instructions
.p2align 2
.globl _asm_metal_add

// Obtain an Objective-C selector. _sel_registerName is a normal C function.
// Metal methods are then called through objc_msgSend with x0=self, x1=selector.
.macro selector label
    adrp x0, \label@PAGE
    add  x0, x0, \label@PAGEOFF
    bl   _sel_registerName
    mov  x1, x0
.endmacro

// Set a failure stage BEFORE a call, so nil/error exits share one cleanup path.
.macro stage value
    mov w9, #\value
    str w9, [sp, #160]
.endmacro

_asm_metal_add:
    // 256-byte, 16-byte-aligned frame. Preserve all callee-saved registers used.
    stp x29, x30, [sp, #-256]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    stp x23, x24, [sp, #48]
    stp x25, x26, [sp, #64]
    stp x27, x28, [sp, #80]

    mov x19, x0                      // input A (CPU pointer)
    mov x20, x1                      // input B (CPU pointer)
    mov x21, x2                      // output (CPU pointer)
    mov x22, x3                      // element count
    mov x23, #0                      // retained MTLDevice
    mov x24, #0                      // retained MTLLibrary
    mov x25, #0                      // retained MTLFunction
    mov x26, #0                      // retained MTLComputePipelineState
    mov x27, #0                      // retained MTLCommandQueue
    mov x28, #0                      // retained output MTLBuffer
    stp xzr, xzr, [sp, #96]          // pool token, retained A buffer
    stp xzr, xzr, [sp, #112]         // retained B buffer, source CFString
    stp xzr, xzr, [sp, #128]         // name CFString, autoreleased NSError
    stp xzr, xzr, [sp, #144]         // autoreleased command buffer, encoder
    str wzr, [sp, #160]              // status

    cbz x22, L_return               // Empty input needs no GPU work.
    stage 1
    cbz x19, L_return
    cbz x20, L_return
    cbz x21, L_return
    lsr x9, x22, #32                 // The shader's index is a 32-bit uint.
    cbnz x9, L_return

    bl _objc_autoreleasePoolPush
    str x0, [sp, #96]

    // A direct call to Metal's exported C function.
    stage 2
    bl _MTLCreateSystemDefaultDevice
    mov x23, x0
    cbz x23, L_cleanup

    // CFString is toll-free bridged to NSString, the type Metal expects.
    stage 3
    mov x0, #0
    adrp x1, L_shader_source@PAGE
    add x1, x1, L_shader_source@PAGEOFF
    mov w2, #0x0100
    movk w2, #0x0800, lsl #16         // kCFStringEncodingUTF8 = 0x08000100
    bl _CFStringCreateWithCString
    str x0, [sp, #120]
    cbz x0, L_cleanup

    // [device newLibraryWithSource:source options:nil error:&error]
    stage 4
    selector L_new_library
    mov x0, x23
    ldr x2, [sp, #120]
    mov x3, #0
    add x4, sp, #136
    bl _objc_msgSend
    mov x24, x0
    cbz x24, L_cleanup

    stage 5
    mov x0, #0
    adrp x1, L_kernel_name@PAGE
    add x1, x1, L_kernel_name@PAGEOFF
    mov w2, #0x0100
    movk w2, #0x0800, lsl #16
    bl _CFStringCreateWithCString
    str x0, [sp, #128]
    cbz x0, L_cleanup
    selector L_new_function
    mov x0, x24
    ldr x2, [sp, #128]
    bl _objc_msgSend
    mov x25, x0
    cbz x25, L_cleanup

    // [device newComputePipelineStateWithFunction:function error:&error]
    stage 6
    selector L_new_pipeline
    mov x0, x23
    mov x2, x25
    str xzr, [sp, #136]
    add x3, sp, #136
    bl _objc_msgSend
    mov x26, x0
    cbz x26, L_cleanup

    stage 7
    selector L_new_queue
    mov x0, x23
    bl _objc_msgSend
    mov x27, x0
    cbz x27, L_cleanup

    // Three shared buffers (MTLResourceStorageModeShared = 0).
    stage 8
    selector L_new_buffer_bytes
    mov x0, x23
    mov x2, x19
    lsl x3, x22, #2                  // byte length = count * sizeof(uint32_t)
    mov x4, #0
    bl _objc_msgSend
    str x0, [sp, #104]
    cbz x0, L_cleanup

    selector L_new_buffer_bytes
    mov x0, x23
    mov x2, x20
    lsl x3, x22, #2
    mov x4, #0
    bl _objc_msgSend
    str x0, [sp, #112]
    cbz x0, L_cleanup

    selector L_new_buffer_length
    mov x0, x23
    lsl x2, x22, #2
    mov x3, #0
    bl _objc_msgSend
    mov x28, x0
    cbz x28, L_cleanup

    stage 9
    selector L_command_buffer
    mov x0, x27
    bl _objc_msgSend
    str x0, [sp, #144]
    cbz x0, L_cleanup
    selector L_compute_encoder
    ldr x0, [sp, #144]
    bl _objc_msgSend
    str x0, [sp, #152]
    cbz x0, L_cleanup

    selector L_set_pipeline
    ldr x0, [sp, #152]
    mov x2, x26
    bl _objc_msgSend

    // [encoder setBuffer:buffer offset:0 atIndex:0/1/2]
    selector L_set_buffer
    ldr x0, [sp, #152]
    ldr x2, [sp, #104]
    mov x3, #0
    mov x4, #0
    bl _objc_msgSend
    selector L_set_buffer
    ldr x0, [sp, #152]
    ldr x2, [sp, #112]
    mov x3, #0
    mov x4, #1
    bl _objc_msgSend
    selector L_set_buffer
    ldr x0, [sp, #152]
    mov x2, x28
    mov x3, #0
    mov x4, #2
    bl _objc_msgSend

    selector L_thread_width
    mov x0, x26
    bl _objc_msgSend
    str x0, [sp, #208]               // group.width = pipeline.threadExecutionWidth
    str x22, [sp, #176]              // grid.width = count
    mov x9, #1
    stp x9, x9, [sp, #184]           // grid.height = grid.depth = 1
    stp x9, x9, [sp, #216]           // group.height = group.depth = 1

    // MTLSize is a 24-byte struct. Apple ARM64 passes each by address here.
    selector L_dispatch
    ldr x0, [sp, #152]
    add x2, sp, #176
    add x3, sp, #208
    bl _objc_msgSend
    selector L_end_encoding
    ldr x0, [sp, #152]
    bl _objc_msgSend

    stage 10
    selector L_commit
    ldr x0, [sp, #144]
    bl _objc_msgSend
    selector L_wait
    ldr x0, [sp, #144]
    bl _objc_msgSend
    selector L_status
    ldr x0, [sp, #144]
    bl _objc_msgSend
    cmp x0, #4                       // MTLCommandBufferStatusCompleted
    b.ne L_cleanup

    stage 11
    selector L_contents
    mov x0, x28
    bl _objc_msgSend
    cbz x0, L_cleanup
    mov x9, x0
L_copy_result:
    ldr w10, [x9], #4
    str w10, [x21], #4
    subs x22, x22, #1
    b.ne L_copy_result
    str wzr, [sp, #160]              // success

L_cleanup:
    // Release only objects returned by create/new; other objects belong to pool.
    mov x0, x28
    bl _objc_release
    ldr x0, [sp, #112]
    bl _objc_release
    ldr x0, [sp, #104]
    bl _objc_release
    mov x0, x27
    bl _objc_release
    mov x0, x26
    bl _objc_release
    mov x0, x25
    bl _objc_release
    mov x0, x24
    bl _objc_release
    mov x0, x23
    bl _objc_release
    ldr x0, [sp, #128]
    cbz x0, L_release_source
    bl _CFRelease
L_release_source:
    ldr x0, [sp, #120]
    cbz x0, L_pop_pool
    bl _CFRelease
L_pop_pool:
    ldr x0, [sp, #96]
    bl _objc_autoreleasePoolPop
L_return:
    ldr w0, [sp, #160]
    ldp x27, x28, [sp, #80]
    ldp x25, x26, [sp, #64]
    ldp x23, x24, [sp, #48]
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #256
    ret

.section __TEXT,__cstring,cstring_literals
L_new_library:       .asciz "newLibraryWithSource:options:error:"
L_new_function:      .asciz "newFunctionWithName:"
L_new_pipeline:      .asciz "newComputePipelineStateWithFunction:error:"
L_new_queue:         .asciz "newCommandQueue"
L_new_buffer_bytes:  .asciz "newBufferWithBytes:length:options:"
L_new_buffer_length: .asciz "newBufferWithLength:options:"
L_command_buffer:    .asciz "commandBuffer"
L_compute_encoder:   .asciz "computeCommandEncoder"
L_set_pipeline:      .asciz "setComputePipelineState:"
L_set_buffer:        .asciz "setBuffer:offset:atIndex:"
L_thread_width:      .asciz "threadExecutionWidth"
L_dispatch:          .asciz "dispatchThreads:threadsPerThreadgroup:"
L_end_encoding:      .asciz "endEncoding"
L_commit:            .asciz "commit"
L_wait:              .asciz "waitUntilCompleted"
L_status:            .asciz "status"
L_contents:          .asciz "contents"
L_kernel_name:       .asciz "vector_add"
L_shader_source:
    .incbin "vector_add.metal"
    .byte 0
.subsections_via_symbols
