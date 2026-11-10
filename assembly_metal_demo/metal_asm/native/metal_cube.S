// Apple ARM64 CPU assembly. All Metal and CAMetalLayer calls live here.
// Context slots (8 bytes each): device, library, vertex, fragment, pipeline,
// queue, depthState, layer, depthTexture, colorTexture, width, height, NSView.
// NSView is borrowed; all other object slots own +1 references.

.section __TEXT,__text,regular,pure_instructions
.p2align 2
.globl _asm_cube_init, _asm_cube_frame, _asm_cube_destroy

.macro cprologue
    stp x29, x30, [sp, #-256]!
    mov x29, sp
    stp x19, x20, [sp, #16]
    stp x21, x22, [sp, #32]
    stp x23, x24, [sp, #48]
    stp x25, x26, [sp, #64]
    stp x27, x28, [sp, #80]
.endmacro
.macro cepilogue
    ldp x27, x28, [sp, #80]
    ldp x25, x26, [sp, #64]
    ldp x23, x24, [sp, #48]
    ldp x21, x22, [sp, #32]
    ldp x19, x20, [sp, #16]
    ldp x29, x30, [sp], #256
    ret
.endmacro
.macro cstage value
    mov w9, #\value
    str w9, [sp, #104]
.endmacro
.macro csel name
    adrp x0, \name@PAGE
    add x0, x0, \name@PAGEOFF
    bl _sel_registerName
    mov x1, x0
.endmacro
// Receiver and argument registers must survive selector lookup (x19..x28).
.macro msg0 receiver, name
    csel \name
    mov x0, \receiver
    bl _objc_msgSend
.endmacro
.macro msg1 receiver, name, arg
    csel \name
    mov x0, \receiver
    mov x2, \arg
    bl _objc_msgSend
.endmacro
.macro allocclass name, dest
    adrp x0, \name@PAGE
    add x0, x0, \name@PAGEOFF
    bl _objc_getClass
    mov \dest, x0
    msg0 \dest, LC_alloc
    mov \dest, x0
    msg0 \dest, LC_init
    mov \dest, x0
.endmacro
// An autoreleased NSString from embedded UTF-8. Clobbers scratch register x25.
.macro string name
    adrp x0, LC_NSString@PAGE
    add x0, x0, LC_NSString@PAGEOFF
    bl _objc_getClass
    mov x25, x0
    csel LC_utf8
    mov x0, x25
    adrp x2, \name@PAGE
    add x2, x2, \name@PAGEOFF
    bl _objc_msgSend
.endmacro

// int asm_cube_init(uint64_t context[13], void *ns_view);
_asm_cube_init:
    cprologue
    mov x19, x0
    mov x20, x1
    stp xzr, xzr, [sp, #112]         // pipeline descriptor, unused
    stp xzr, xzr, [sp, #128]         // depth descriptor, unused
    str xzr, [sp, #144]             // NSError *
    bl _objc_autoreleasePoolPush
    str x0, [sp, #96]
    cstage 1
    bl _MTLCreateSystemDefaultDevice
    str x0, [x19, #0]
    mov x23, x0
    cbz x23, LC_init_done
    cstage 2
    string LC_shader
    mov x26, x0
    cbz x26, LC_init_done
    csel LC_new_library
    mov x0, x23
    mov x2, x26
    mov x3, #0
    add x4, sp, #144
    bl _objc_msgSend
    str x0, [x19, #8]
    mov x26, x0
    cbz x26, LC_init_done
    string LC_vertex_name
    mov x27, x0
    msg1 x26, LC_new_function, x27
    str x0, [x19, #16]
    cbz x0, LC_init_done
    string LC_fragment_name
    mov x27, x0
    msg1 x26, LC_new_function, x27
    str x0, [x19, #24]
    cbz x0, LC_init_done

    cstage 3
    allocclass LC_PipelineDescriptor, x24
    str x24, [sp, #112]
    cbz x24, LC_init_done
    ldr x26, [x19, #16]
    msg1 x24, LC_set_vertex, x26
    ldr x26, [x19, #24]
    msg1 x24, LC_set_fragment, x26
    msg1 x24, LC_set_depth_format, #252 // MTLPixelFormatDepth32Float
    msg0 x24, LC_color_attachments
    mov x26, x0
    msg1 x26, LC_subscript, #0
    mov x26, x0
    msg1 x26, LC_set_pixel_format, #80 // MTLPixelFormatBGRA8Unorm
    csel LC_new_pipeline
    mov x0, x23
    mov x2, x24
    add x3, sp, #144
    bl _objc_msgSend
    str x0, [x19, #32]
    cbz x0, LC_init_done
    allocclass LC_DepthDescriptor, x24
    str x24, [sp, #128]
    cbz x24, LC_init_done
    msg1 x24, LC_set_depth_compare, #1 // MTLCompareFunctionLess
    msg1 x24, LC_set_depth_write, #1
    msg1 x23, LC_new_depth_state, x24
    str x0, [x19, #48]
    cbz x0, LC_init_done
    msg0 x23, LC_new_queue
    str x0, [x19, #40]
    cbz x0, LC_init_done
    cbz x20, LC_init_success          // null view selects offscreen rendering
    allocclass LC_CAMetalLayer, x24
    str x24, [x19, #56]
    cbz x24, LC_init_done
    msg1 x24, LC_set_device, x23
    msg1 x24, LC_set_pixel_format, #80
    msg1 x24, LC_set_framebuffer_only, #1
    msg1 x20, LC_set_wants_layer, #1
    msg1 x20, LC_set_layer, x24
    str x20, [x19, #96]             // borrow NSView only after attaching layer
LC_init_success:
    cstage 0
LC_init_done:
    ldr x0, [sp, #128]
    bl _objc_release
    ldr x0, [sp, #112]
    bl _objc_release
    ldr x0, [sp, #96]
    bl _objc_autoreleasePoolPop
    ldr w0, [sp, #104]
    cepilogue

// int asm_cube_frame(context*, size_t width, size_t height, uint8_t *bgra, float angle);
// x0..x3 are integer arguments; s0 is independently allocated by the ARM64 ABI.
// bgra is null for presentation, or width*height*4 bytes for offscreen readback.
_asm_cube_frame:
    cprologue
    mov x19, x0
    mov x20, x1
    mov x21, x2
    mov x22, x3
    str s0, [sp, #160]              // float4 uniforms: angle, aspect, 0, 0
    ucvtf s1, x20
    ucvtf s2, x21
    fdiv s1, s1, s2
    str s1, [sp, #164]
    str xzr, [sp, #168]
    stp xzr, xzr, [sp, #112]         // owned pass descriptor, borrowed drawable
    str xzr, [sp, #128]             // owned texture descriptor
    bl _objc_autoreleasePoolPush
    str x0, [sp, #96]
    ldr x23, [x19, #0]
    cstage 4
    ldr x9, [x19, #80]
    cmp x9, x20
    b.ne LC_resize
    ldr x9, [x19, #88]
    cmp x9, x21
    b.ne LC_resize
    ldr x9, [x19, #64]
    cbnz x9, LC_textures_ready
LC_resize:
    ldr x0, [x19, #64]
    bl _objc_release
    str xzr, [x19, #64]
    ldr x0, [x19, #72]
    bl _objc_release
    str xzr, [x19, #72]
    stp xzr, xzr, [x19, #80]         // invalidate dimensions if allocation fails
    allocclass LC_TextureDescriptor, x24
    str x24, [sp, #128]
    cbz x24, LC_frame_done
    msg1 x24, LC_set_texture_type, #2 // MTLTextureType2D
    msg1 x24, LC_set_width, x20
    msg1 x24, LC_set_height, x21
    msg1 x24, LC_set_pixel_format, #252
    msg1 x24, LC_set_storage, #2      // private depth attachment
    msg1 x24, LC_set_usage, #4        // MTLTextureUsageRenderTarget
    msg1 x23, LC_new_texture, x24
    str x0, [x19, #64]
    cbz x0, LC_frame_done
    ldr x9, [x19, #56]
    cbnz x9, LC_resize_done
    msg1 x24, LC_set_pixel_format, #80
    msg1 x24, LC_set_storage, #0      // shared color attachment for CPU readback
    msg1 x23, LC_new_texture, x24
    str x0, [x19, #72]
    cbz x0, LC_frame_done
LC_resize_done:
    stp x20, x21, [x19, #80]
LC_textures_ready:
    cstage 5
    ldr x24, [x19, #56]
    cbz x24, LC_offscreen_target
    // CGSize is a homogeneous aggregate of two doubles, passed in d0 and d1.
    csel LC_set_drawable_size
    mov x0, x24
    ucvtf d0, x20
    ucvtf d1, x21
    bl _objc_msgSend
    msg0 x24, LC_next_drawable
    str x0, [sp, #120]
    cbz x0, LC_frame_skipped          // minimized/temporarily unavailable surface
    mov x25, x0
    msg0 x25, LC_texture
    mov x26, x0
    b LC_target_ready
LC_offscreen_target:
    ldr x26, [x19, #72]
LC_target_ready:
    cbz x26, LC_frame_done
    allocclass LC_PassDescriptor, x24
    str x24, [sp, #112]
    cbz x24, LC_frame_done
    msg0 x24, LC_color_attachments
    mov x25, x0
    msg1 x25, LC_subscript, #0
    mov x25, x0
    msg1 x25, LC_set_texture, x26
    msg1 x25, LC_set_load, #2         // clear
    msg1 x25, LC_set_store, #1        // store
    // MTLClearColor is four doubles: an HFA in d0..d3, despite its 32-byte size.
    csel LC_set_clear_color
    mov x0, x25
    fmov d0, #0.25
    fmov d1, #0.5
    fmov d2, #0.125
    fmul d0, d0, d2                 // .03125: below fmov immediate range
    fmul d1, d1, d2                 // .0625
    fmov d3, #1.0
    bl _objc_msgSend
    msg0 x24, LC_depth_attachment
    mov x25, x0
    ldr x27, [x19, #64]
    msg1 x25, LC_set_texture, x27
    msg1 x25, LC_set_load, #2
    msg1 x25, LC_set_store, #0        // discard depth after this pass
    csel LC_set_clear_depth
    mov x0, x25
    fmov d0, #1.0
    bl _objc_msgSend
    ldr x25, [x19, #40]
    msg0 x25, LC_command_buffer
    mov x27, x0
    cbz x27, LC_frame_done
    msg1 x27, LC_render_encoder, x24
    mov x28, x0
    cbz x28, LC_frame_done
    ldr x25, [x19, #32]
    msg1 x28, LC_set_render_pipeline, x25
    ldr x25, [x19, #48]
    msg1 x28, LC_set_depth_state, x25
    csel LC_set_vertex_bytes
    mov x0, x28
    add x2, sp, #160
    mov x3, #16
    mov x4, #0
    bl _objc_msgSend
    // 12 triangles / 36 vertices; vertex positions and colors are in the shader.
    csel LC_draw
    mov x0, x28
    mov x2, #3                      // MTLPrimitiveTypeTriangle
    mov x3, #0
    mov x4, #36
    bl _objc_msgSend
    msg0 x28, LC_end_encoding
    ldr x25, [sp, #120]
    cbz x25, LC_submit
    msg1 x27, LC_present, x25
LC_submit:
    cstage 6
    msg0 x27, LC_commit
    msg0 x27, LC_wait
    msg0 x27, LC_status
    cmp x0, #4                      // MTLCommandBufferStatusCompleted
    b.ne LC_frame_done
    cbz x22, LC_frame_skipped
    // MTLRegion is 48 bytes (origin.xyz, size.whd), passed indirectly in x4.
    stp xzr, xzr, [sp, #192]
    str xzr, [sp, #208]
    stp x20, x21, [sp, #216]
    mov x9, #1
    str x9, [sp, #232]
    csel LC_get_bytes
    mov x0, x26
    mov x2, x22
    lsl x3, x20, #2
    add x4, sp, #192
    mov x5, #0
    bl _objc_msgSend
LC_frame_skipped:
    cstage 0
LC_frame_done:
    ldr x0, [sp, #128]
    bl _objc_release
    ldr x0, [sp, #112]
    bl _objc_release
    ldr x0, [sp, #96]
    bl _objc_autoreleasePoolPop
    ldr w0, [sp, #104]
    cepilogue

// void asm_cube_destroy(context*); safe for a partially initialized context.
_asm_cube_destroy:
    cprologue
    mov x19, x0
    bl _objc_autoreleasePoolPush
    str x0, [sp, #96]
    ldr x20, [x19, #96]
    cbz x20, LC_release_objects
    msg1 x20, LC_set_layer, #0
    str xzr, [x19, #96]
LC_release_objects:
    mov x20, #72                    // release objects in reverse slot order
LC_release_loop:
    ldr x0, [x19, x20]
    bl _objc_release
    str xzr, [x19, x20]
    subs x20, x20, #8
    b.ge LC_release_loop
    stp xzr, xzr, [x19, #80]
    ldr x0, [sp, #96]
    bl _objc_autoreleasePoolPop
    cepilogue

.section __TEXT,__cstring,cstring_literals
LC_NSString:              .asciz "NSString"
LC_PipelineDescriptor:    .asciz "MTLRenderPipelineDescriptor"
LC_DepthDescriptor:       .asciz "MTLDepthStencilDescriptor"
LC_TextureDescriptor:     .asciz "MTLTextureDescriptor"
LC_PassDescriptor:        .asciz "MTLRenderPassDescriptor"
LC_CAMetalLayer:          .asciz "CAMetalLayer"
LC_alloc:                 .asciz "alloc"
LC_init:                  .asciz "init"
LC_utf8:                  .asciz "stringWithUTF8String:"
LC_new_library:           .asciz "newLibraryWithSource:options:error:"
LC_new_function:          .asciz "newFunctionWithName:"
LC_set_vertex:            .asciz "setVertexFunction:"
LC_set_fragment:          .asciz "setFragmentFunction:"
LC_set_depth_format:      .asciz "setDepthAttachmentPixelFormat:"
LC_color_attachments:     .asciz "colorAttachments"
LC_subscript:             .asciz "objectAtIndexedSubscript:"
LC_set_pixel_format:      .asciz "setPixelFormat:"
LC_new_pipeline:          .asciz "newRenderPipelineStateWithDescriptor:error:"
LC_set_depth_compare:     .asciz "setDepthCompareFunction:"
LC_set_depth_write:       .asciz "setDepthWriteEnabled:"
LC_new_depth_state:       .asciz "newDepthStencilStateWithDescriptor:"
LC_new_queue:             .asciz "newCommandQueue"
LC_set_device:            .asciz "setDevice:"
LC_set_framebuffer_only:  .asciz "setFramebufferOnly:"
LC_set_wants_layer:       .asciz "setWantsLayer:"
LC_set_layer:             .asciz "setLayer:"
LC_set_texture_type:      .asciz "setTextureType:"
LC_set_width:             .asciz "setWidth:"
LC_set_height:            .asciz "setHeight:"
LC_set_storage:           .asciz "setStorageMode:"
LC_set_usage:             .asciz "setUsage:"
LC_new_texture:           .asciz "newTextureWithDescriptor:"
LC_set_drawable_size:     .asciz "setDrawableSize:"
LC_next_drawable:         .asciz "nextDrawable"
LC_texture:               .asciz "texture"
LC_set_texture:           .asciz "setTexture:"
LC_set_load:              .asciz "setLoadAction:"
LC_set_store:             .asciz "setStoreAction:"
LC_set_clear_color:       .asciz "setClearColor:"
LC_depth_attachment:      .asciz "depthAttachment"
LC_set_clear_depth:       .asciz "setClearDepth:"
LC_command_buffer:        .asciz "commandBuffer"
LC_render_encoder:        .asciz "renderCommandEncoderWithDescriptor:"
LC_set_render_pipeline:   .asciz "setRenderPipelineState:"
LC_set_depth_state:        .asciz "setDepthStencilState:"
LC_set_vertex_bytes:       .asciz "setVertexBytes:length:atIndex:"
LC_draw:                  .asciz "drawPrimitives:vertexStart:vertexCount:"
LC_end_encoding:          .asciz "endEncoding"
LC_present:               .asciz "presentDrawable:"
LC_commit:                .asciz "commit"
LC_wait:                  .asciz "waitUntilCompleted"
LC_status:                .asciz "status"
LC_get_bytes:             .asciz "getBytes:bytesPerRow:fromRegion:mipmapLevel:"
LC_vertex_name:           .asciz "cube_vertex"
LC_fragment_name:         .asciz "cube_fragment"
LC_shader:
    .incbin "cube.metal"
    .byte 0
.subsections_via_symbols
