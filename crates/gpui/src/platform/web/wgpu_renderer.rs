use super::WgpuAtlas;
use crate::{
    Background, Bounds, DevicePixels, GpuSpecs, MonochromeSprite, Path, PolychromeSprite,
    PrimitiveBatch, Quad, ScaledPixels, Scene, Shadow, Size, Underline,
    get_gamma_correction_ratios,
};
use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

const MSAA_SAMPLE_COUNTS: [u32; 3] = [4, 2, 1];

fn slice_to_bytes<T>(data: &[T]) -> &[u8] {
    let ptr = data.as_ptr() as *const u8;
    let len = std::mem::size_of_val(data);
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct GlobalParams {
    viewport_size: [f32; 2],
    premultiplied_alpha: u32,
    pad: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PathRasterizationVertex {
    xy_position: [f32; 2],
    st_position: [f32; 2],
    color: GpuBackground,
    bounds: GpuBounds,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuBackground {
    tag: u32,
    color_space: u32,
    solid: [f32; 4],  // Hsla as 4 floats
    gradient_angle_or_pattern_height: f32,
    colors: [[f32; 5]; 2],  // 2x LinearColorStop (Hsla + percentage)
    pad: u32,
}

impl From<Background> for GpuBackground {
    fn from(bg: Background) -> Self {
        unsafe { std::mem::transmute(bg) }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuBounds {
    origin: [f32; 2],
    size: [f32; 2],
}

impl From<Bounds<ScaledPixels>> for GpuBounds {
    fn from(b: Bounds<ScaledPixels>) -> Self {
        Self {
            origin: [b.origin.x.0, b.origin.y.0],
            size: [b.size.width.0, b.size.height.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuCorners {
    top_left: f32,
    top_right: f32,
    bottom_right: f32,
    bottom_left: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuEdges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuQuad {
    bounds: GpuBounds,
    content_mask: GpuBounds,
    background: GpuBackground,
    border_color: [f32; 4],
    corner_radii: GpuCorners,
    border_widths: GpuEdges,
}

impl From<&Quad> for GpuQuad {
    fn from(quad: &Quad) -> Self {
        Self {
            bounds: GpuBounds {
                origin: [quad.bounds.origin.x.0, quad.bounds.origin.y.0],
                size: [quad.bounds.size.width.0, quad.bounds.size.height.0],
            },
            content_mask: GpuBounds {
                origin: [quad.content_mask.bounds.origin.x.0, quad.content_mask.bounds.origin.y.0],
                size: [quad.content_mask.bounds.size.width.0, quad.content_mask.bounds.size.height.0],
            },
            background: quad.background.clone().into(),
            border_color: [
                quad.border_color.h,
                quad.border_color.s,
                quad.border_color.l,
                quad.border_color.a,
            ],
            corner_radii: GpuCorners {
                top_left: quad.corner_radii.top_left.0,
                top_right: quad.corner_radii.top_right.0,
                bottom_right: quad.corner_radii.bottom_right.0,
                bottom_left: quad.corner_radii.bottom_left.0,
            },
            border_widths: GpuEdges {
                top: quad.border_widths.top.0,
                right: quad.border_widths.right.0,
                bottom: quad.border_widths.bottom.0,
                left: quad.border_widths.left.0,
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuShadow {
    blur_radius: f32,
    _pad: [f32; 3],
    bounds: GpuBounds,
    corner_radii: GpuCorners,
    content_mask: GpuBounds,
    color: [f32; 4],
}

impl From<&Shadow> for GpuShadow {
    fn from(shadow: &Shadow) -> Self {
        Self {
            blur_radius: shadow.blur_radius.0,
            _pad: [0.0; 3],
            bounds: GpuBounds {
                origin: [shadow.bounds.origin.x.0, shadow.bounds.origin.y.0],
                size: [shadow.bounds.size.width.0, shadow.bounds.size.height.0],
            },
            corner_radii: GpuCorners {
                top_left: shadow.corner_radii.top_left.0,
                top_right: shadow.corner_radii.top_right.0,
                bottom_right: shadow.corner_radii.bottom_right.0,
                bottom_left: shadow.corner_radii.bottom_left.0,
            },
            content_mask: GpuBounds {
                origin: [shadow.content_mask.bounds.origin.x.0, shadow.content_mask.bounds.origin.y.0],
                size: [shadow.content_mask.bounds.size.width.0, shadow.content_mask.bounds.size.height.0],
            },
            color: [shadow.color.h, shadow.color.s, shadow.color.l, shadow.color.a],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuUnderline {
    bounds: GpuBounds,
    content_mask: GpuBounds,
    color: [f32; 4],
    thickness: f32,
    wavy: u32,
    _pad: [f32; 2],
}

impl From<&Underline> for GpuUnderline {
    fn from(underline: &Underline) -> Self {
        Self {
            bounds: GpuBounds {
                origin: [underline.bounds.origin.x.0, underline.bounds.origin.y.0],
                size: [underline.bounds.size.width.0, underline.bounds.size.height.0],
            },
            content_mask: GpuBounds {
                origin: [underline.content_mask.bounds.origin.x.0, underline.content_mask.bounds.origin.y.0],
                size: [underline.content_mask.bounds.size.width.0, underline.content_mask.bounds.size.height.0],
            },
            color: [
                underline.color.h,
                underline.color.s,
                underline.color.l,
                underline.color.a,
            ],
            thickness: underline.thickness.0,
            wavy: underline.wavy,
            _pad: [0.0; 2],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuAtlasTile {
    bounds_origin: [i32; 2],
    bounds_size: [i32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuTransformationMatrix {
    rotation_scale: [[f32; 2]; 2],
    translation: [f32; 2],
    _pad: [f32; 2],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuMonochromeSprite {
    bounds: GpuBounds,
    content_mask: GpuBounds,
    color: [f32; 4],
    tile: GpuAtlasTile,
    transformation: GpuTransformationMatrix,
}

impl From<&MonochromeSprite> for GpuMonochromeSprite {
    fn from(sprite: &MonochromeSprite) -> Self {
        Self {
            bounds: GpuBounds {
                origin: [sprite.bounds.origin.x.0, sprite.bounds.origin.y.0],
                size: [sprite.bounds.size.width.0, sprite.bounds.size.height.0],
            },
            content_mask: GpuBounds {
                origin: [sprite.content_mask.bounds.origin.x.0, sprite.content_mask.bounds.origin.y.0],
                size: [sprite.content_mask.bounds.size.width.0, sprite.content_mask.bounds.size.height.0],
            },
            color: [sprite.color.h, sprite.color.s, sprite.color.l, sprite.color.a],
            tile: GpuAtlasTile {
                bounds_origin: [sprite.tile.bounds.origin.x.0, sprite.tile.bounds.origin.y.0],
                bounds_size: [sprite.tile.bounds.size.width.0, sprite.tile.bounds.size.height.0],
            },
            transformation: GpuTransformationMatrix {
                rotation_scale: sprite.transformation.rotation_scale,
                translation: sprite.transformation.translation,
                _pad: [0.0; 2],
            },
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable, Default)]
struct GpuPolychromeSprite {
    grayscale: u32,
    _pad: u32,
    opacity: f32,
    _pad2: f32,
    bounds: GpuBounds,
    content_mask: GpuBounds,
    corner_radii: GpuCorners,
    tile: GpuAtlasTile,
}

impl From<&PolychromeSprite> for GpuPolychromeSprite {
    fn from(sprite: &PolychromeSprite) -> Self {
        Self {
            grayscale: if sprite.grayscale { 1 } else { 0 },
            _pad: 0,
            opacity: sprite.opacity,
            _pad2: 0.0,
            bounds: GpuBounds {
                origin: [sprite.bounds.origin.x.0, sprite.bounds.origin.y.0],
                size: [sprite.bounds.size.width.0, sprite.bounds.size.height.0],
            },
            content_mask: GpuBounds {
                origin: [sprite.content_mask.bounds.origin.x.0, sprite.content_mask.bounds.origin.y.0],
                size: [sprite.content_mask.bounds.size.width.0, sprite.content_mask.bounds.size.height.0],
            },
            corner_radii: GpuCorners {
                top_left: sprite.corner_radii.top_left.0,
                top_right: sprite.corner_radii.top_right.0,
                bottom_right: sprite.corner_radii.bottom_right.0,
                bottom_left: sprite.corner_radii.bottom_left.0,
            },
            tile: GpuAtlasTile {
                bounds_origin: [sprite.tile.bounds.origin.x.0, sprite.tile.bounds.origin.y.0],
                bounds_size: [sprite.tile.bounds.size.width.0, sprite.tile.bounds.size.height.0],
            },
        }
    }
}

fn quad_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<GpuQuad>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // bounds.origin (location 0)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // bounds.size (location 1)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            // content_mask.origin (location 2)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 2,
            },
            // content_mask.size (location 3)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 3,
            },
            // background.tag, color_space (location 4)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32x2,
                offset: 32,
                shader_location: 4,
            },
            // background.solid (location 5)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 40,
                shader_location: 5,
            },
            // background.gradient_angle_or_pattern_height + colors[0].color (location 6)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 56,
                shader_location: 6,
            },
            // colors[0].percentage + colors[1].color.hsl (location 7)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 72,
                shader_location: 7,
            },
            // colors[1].color.a + colors[1].percentage + pad (location 8)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 88,
                shader_location: 8,
            },
            // border_color (location 9)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 104,
                shader_location: 9,
            },
            // corner_radii (location 10)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 120,
                shader_location: 10,
            },
            // border_widths (location 11)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 136,
                shader_location: 11,
            },
        ],
    }
}

fn shadow_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<GpuShadow>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // blur_radius + pad (location 0)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 0,
                shader_location: 0,
            },
            // bounds.origin (location 1)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 1,
            },
            // bounds.size (location 2)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 2,
            },
            // corner_radii (location 3)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 3,
            },
            // content_mask.origin (location 4)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 48,
                shader_location: 4,
            },
            // content_mask.size (location 5)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 56,
                shader_location: 5,
            },
            // color (location 6)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 64,
                shader_location: 6,
            },
        ],
    }
}

fn underline_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<GpuUnderline>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // bounds.origin (location 0)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // bounds.size (location 1)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            // content_mask.origin (location 2)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 2,
            },
            // content_mask.size (location 3)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 3,
            },
            // color (location 4)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 4,
            },
            // thickness, wavy, pad (location 5)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 5,
            },
        ],
    }
}

fn mono_sprite_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<GpuMonochromeSprite>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // bounds.origin (location 0)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // bounds.size (location 1)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            // content_mask.origin (location 2)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 2,
            },
            // content_mask.size (location 3)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 3,
            },
            // color (location 4)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 32,
                shader_location: 4,
            },
            // tile.bounds_origin (location 5)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Sint32x2,
                offset: 48,
                shader_location: 5,
            },
            // tile.bounds_size (location 6)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Sint32x2,
                offset: 56,
                shader_location: 6,
            },
            // transformation.rotation_scale row 0 (location 7)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 64,
                shader_location: 7,
            },
            // transformation.rotation_scale row 1 (location 8)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 72,
                shader_location: 8,
            },
            // transformation.translation + pad (location 9)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 80,
                shader_location: 9,
            },
        ],
    }
}

fn poly_sprite_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<GpuPolychromeSprite>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // grayscale, pad (location 0)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32x2,
                offset: 0,
                shader_location: 0,
            },
            // opacity, pad2 (location 1)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            // bounds.origin (location 2)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 16,
                shader_location: 2,
            },
            // bounds.size (location 3)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 24,
                shader_location: 3,
            },
            // content_mask.origin (location 4)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 32,
                shader_location: 4,
            },
            // content_mask.size (location 5)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 40,
                shader_location: 5,
            },
            // corner_radii (location 6)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 48,
                shader_location: 6,
            },
            // tile.bounds_origin (location 7)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Sint32x2,
                offset: 64,
                shader_location: 7,
            },
            // tile.bounds_size (location 8)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Sint32x2,
                offset: 72,
                shader_location: 8,
            },
        ],
    }
}

fn path_rasterization_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<PathRasterizationVertex>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &[
            // xy_position (location 0)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // st_position (location 1)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
            // color.tag, color_space (location 2)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Uint32x2,
                offset: 16,
                shader_location: 2,
            },
            // color.solid (location 3)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 24,
                shader_location: 3,
            },
            // color.gradient_angle + colors[0].color (location 4)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 40,
                shader_location: 4,
            },
            // colors[0].percentage + colors[1].color.hsl (location 5)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 56,
                shader_location: 5,
            },
            // colors[1].color.a + colors[1].percentage + pad (location 6)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: 72,
                shader_location: 6,
            },
            // bounds.origin (location 7)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 88,
                shader_location: 7,
            },
            // bounds.size (location 8)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 96,
                shader_location: 8,
            },
        ],
    }
}

fn path_sprite_vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<PathSprite>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &[
            // bounds.origin (location 0)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            },
            // bounds.size (location 1)
            wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 8,
                shader_location: 1,
            },
        ],
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PathSprite {
    bounds: GpuBounds,
}

struct WgpuPipelines {
    quads: wgpu::RenderPipeline,
    shadows: wgpu::RenderPipeline,
    underlines: wgpu::RenderPipeline,
    mono_sprites: wgpu::RenderPipeline,
    poly_sprites: wgpu::RenderPipeline,
    path_rasterization: wgpu::RenderPipeline,
    paths: wgpu::RenderPipeline,
}

pub struct WgpuRenderer {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    surface: wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    pipelines: WgpuPipelines,
    atlas: Arc<WgpuAtlas>,
    atlas_sampler: wgpu::Sampler,
    bind_group_layout: wgpu::BindGroupLayout,
    rendering_parameters: RenderingParameters,
    dummy_texture: wgpu::Texture,
    path_intermediate_texture: Option<wgpu::Texture>,
    path_intermediate_texture_view: Option<wgpu::TextureView>,
    path_intermediate_msaa_texture: Option<wgpu::Texture>,
    path_intermediate_msaa_texture_view: Option<wgpu::TextureView>,
    path_sample_count: u32,
}

#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
struct SpriteParams {
    gamma_ratios: [f32; 4],
    grayscale_enhanced_contrast: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

#[derive(Clone)]
struct RenderingParameters {
    sprite_params: SpriteParams,
}

impl RenderingParameters {
    fn new() -> Self {
        let gamma = 1.8; // Default gamma for web
        Self {
            sprite_params: SpriteParams {
                gamma_ratios: get_gamma_correction_ratios(gamma),
                grayscale_enhanced_contrast: 1.0,
                _pad0: 0.0,
                _pad1: 0.0,
                _pad2: 0.0,
            },
        }
    }
}

impl WgpuRenderer {
    pub async fn new(
        canvas: web_sys::HtmlCanvasElement,
        size: Size<DevicePixels>,
        transparent: bool,
    ) -> anyhow::Result<Self> {
        // Try WebGPU first, then fall back to WebGL
        let backends_to_try = [
            ("WebGPU", wgpu::Backends::BROWSER_WEBGPU),
            ("WebGL", wgpu::Backends::GL),
        ];

        let mut last_error = None;

        for (backend_name, backend) in backends_to_try {
            web_sys::console::log_1(&format!("Trying {} backend...", backend_name).into());

            let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
                backends: backend,
                ..Default::default()
            });

            let surface_target = wgpu::SurfaceTarget::Canvas(canvas.clone());
            let surface = match instance.create_surface(surface_target) {
                Ok(s) => s,
                Err(e) => {
                    web_sys::console::warn_1(&format!("✗ {} backend: failed to create surface: {}", backend_name, e).into());
                    last_error = Some(format!("{} surface creation failed: {}", backend_name, e));
                    continue;
                }
            };

            let adapter = match instance
                .request_adapter(&wgpu::RequestAdapterOptions {
                    power_preference: wgpu::PowerPreference::HighPerformance,
                    compatible_surface: Some(&surface),
                    force_fallback_adapter: false,
                })
                .await
            {
                Some(a) => a,
                None => {
                    web_sys::console::warn_1(&format!("✗ {} backend: no suitable adapter found", backend_name).into());
                    last_error = Some(format!("{} no suitable adapter", backend_name));
                    continue;
                }
            };

            let limits = if backend == wgpu::Backends::GL {
                wgpu::Limits::downlevel_webgl2_defaults()
            } else {
                wgpu::Limits::default()
            };

            let (device, queue) = match adapter
                .request_device(
                    &wgpu::DeviceDescriptor {
                        label: Some("gpui"),
                        required_features: wgpu::Features::empty(),
                        required_limits: limits,
                        memory_hints: Default::default(),
                    },
                    None,
                )
                .await
            {
                Ok(dq) => dq,
                Err(e) => {
                    web_sys::console::warn_1(&format!("✗ {} backend: failed to request device: {}", backend_name, e).into());
                    last_error = Some(format!("{} device request failed: {}", backend_name, e));
                    continue;
                }
            };

            let device = Arc::new(device);
            let queue = Arc::new(queue);

            let surface_caps = surface.get_capabilities(&adapter);
            // Prefer non-sRGB format to match Metal's BGRA8Unorm behavior.
            // The shader outputs sRGB-space colors directly, so using an sRGB framebuffer
            // would cause double gamma correction (washed out colors).
            let surface_format = surface_caps
                .formats
                .iter()
                .find(|f| !f.is_srgb())
                .copied()
                .unwrap_or(surface_caps.formats[0]);

            let alpha_mode = if transparent {
                wgpu::CompositeAlphaMode::PreMultiplied
            } else {
                wgpu::CompositeAlphaMode::Opaque
            };

            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: surface_format,
                width: size.width.0 as u32,
                height: size.height.0 as u32,
                present_mode: wgpu::PresentMode::Fifo,
                desired_maximum_frame_latency: 2,
                alpha_mode,
                view_formats: vec![],
            };
            surface.configure(&device, &surface_config);

            let bind_group_layout = create_bind_group_layout(&device);

            // Try MSAA sample counts in order: 4x, 2x, 1x
            let mut path_sample_count = 1;
            for &sample_count in &MSAA_SAMPLE_COUNTS {
                if sample_count == 1 {
                    path_sample_count = 1;
                    break;
                }

                // Test if this sample count is supported by creating a test texture
                let test_result = device.create_texture(&wgpu::TextureDescriptor {
                    label: Some("msaa test"),
                    size: wgpu::Extent3d {
                        width: 16,
                        height: 16,
                        depth_or_array_layers: 1,
                    },
                    mip_level_count: 1,
                    sample_count,
                    dimension: wgpu::TextureDimension::D2,
                    format: surface_format,
                    usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                    view_formats: &[],
                });

                // If texture creation succeeded, this sample count is supported
                drop(test_result);
                path_sample_count = sample_count;
                break;
            }

            let pipelines = create_pipelines(&device, surface_format, &bind_group_layout, path_sample_count);

            let atlas = Arc::new(WgpuAtlas::new(device.clone(), queue.clone()));
            let atlas_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
                label: Some("atlas sampler"),
                mag_filter: wgpu::FilterMode::Linear,
                min_filter: wgpu::FilterMode::Linear,
                ..Default::default()
            });

            let rendering_parameters = RenderingParameters::new();

            let dummy_texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("dummy texture"),
                size: wgpu::Extent3d {
                    width: 1,
                    height: 1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::R8Unorm,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            web_sys::console::log_1(&format!("✓ {} backend initialized with {}x MSAA", backend_name, path_sample_count).into());

            return Ok(Self {
                device,
                queue,
                surface,
                surface_config,
                pipelines,
                atlas,
                atlas_sampler,
                bind_group_layout,
                rendering_parameters,
                dummy_texture,
                path_intermediate_texture: None,
                path_intermediate_texture_view: None,
                path_intermediate_msaa_texture: None,
                path_intermediate_msaa_texture_view: None,
                path_sample_count,
            });
        }

        Err(anyhow::anyhow!(
            "Failed to initialize any graphics backend. Last error: {}",
            last_error.unwrap_or_else(|| "unknown".to_string())
        ))
    }

    pub fn update_drawable_size(&mut self, size: Size<DevicePixels>) {
        if size.width.0 as u32 != self.surface_config.width
            || size.height.0 as u32 != self.surface_config.height
        {
            self.surface_config.width = size.width.0 as u32;
            self.surface_config.height = size.height.0 as u32;
            self.surface.configure(&self.device, &self.surface_config);
            self.path_intermediate_texture = None;
            self.path_intermediate_texture_view = None;
            self.path_intermediate_msaa_texture = None;
            self.path_intermediate_msaa_texture_view = None;
        }
    }

    fn ensure_path_intermediate_texture(&mut self) {
        if self.path_intermediate_texture.is_some() {
            return;
        }

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("path intermediate texture"),
            size: wgpu::Extent3d {
                width: self.surface_config.width,
                height: self.surface_config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: self.surface_config.format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        self.path_intermediate_texture = Some(texture);
        self.path_intermediate_texture_view = Some(view);

        if self.path_sample_count > 1 {
            let msaa_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("path intermediate MSAA texture"),
                size: wgpu::Extent3d {
                    width: self.surface_config.width,
                    height: self.surface_config.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: self.path_sample_count,
                dimension: wgpu::TextureDimension::D2,
                format: self.surface_config.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                view_formats: &[],
            });

            let msaa_view = msaa_texture.create_view(&wgpu::TextureViewDescriptor::default());
            self.path_intermediate_msaa_texture = Some(msaa_texture);
            self.path_intermediate_msaa_texture_view = Some(msaa_view);
        }
    }

    pub fn sprite_atlas(&self) -> &Arc<WgpuAtlas> {
        &self.atlas
    }

    pub fn gpu_specs(&self) -> GpuSpecs {
        GpuSpecs {
            is_software_emulated: false,
            device_name: "WebGPU".to_string(),
            driver_name: "Browser".to_string(),
            driver_info: "WebGPU API".to_string(),
        }
    }

    pub fn draw(&mut self, scene: &Scene) {
        self.atlas.before_frame();

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                self.surface.configure(&self.device, &self.surface_config);
                return;
            }
            Err(e) => {
                log::error!("Failed to acquire next swap chain texture: {:?}", e);
                return;
            }
        };

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render encoder"),
        });

        let globals = GlobalParams {
            viewport_size: [
                self.surface_config.width as f32,
                self.surface_config.height as f32,
            ],
            premultiplied_alpha: match self.surface_config.alpha_mode {
                wgpu::CompositeAlphaMode::PreMultiplied => 1,
                _ => 0,
            },
            pad: 0,
        };

        let mut is_first_pass = true;

        for batch in scene.batches() {
            match batch {
                PrimitiveBatch::Paths(paths) => {
                    if paths.is_empty() {
                        continue;
                    }

                    self.ensure_path_intermediate_texture();
                    self.draw_paths_to_intermediate(&mut encoder, paths, &globals);

                    let load_op = if is_first_pass {
                        is_first_pass = false;
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    } else {
                        wgpu::LoadOp::Load
                    };

                    let Some(ref intermediate_view) = self.path_intermediate_texture_view else {
                        continue;
                    };

                    {
                        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                            label: Some("paths copy pass"),
                            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                                view: &view,
                                resolve_target: None,
                                ops: wgpu::Operations {
                                    load: load_op,
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });

                        self.draw_path_sprites(&mut render_pass, paths, intermediate_view, &globals);
                    }
                }
                _ => {
                    let load_op = if is_first_pass {
                        is_first_pass = false;
                        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
                    } else {
                        wgpu::LoadOp::Load
                    };

                    let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                        label: Some("main render pass"),
                        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: wgpu::Operations {
                                load: load_op,
                                store: wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: None,
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });

                    match batch {
                        PrimitiveBatch::Quads(quads) => {
                            self.draw_quads(&mut render_pass, quads, &globals);
                        }
                        PrimitiveBatch::Shadows(shadows) => {
                            self.draw_shadows(&mut render_pass, shadows, &globals);
                        }
                        PrimitiveBatch::Underlines(underlines) => {
                            self.draw_underlines(&mut render_pass, underlines, &globals);
                        }
                        PrimitiveBatch::MonochromeSprites { texture_id, sprites } => {
                            self.draw_mono_sprites(&mut render_pass, texture_id, sprites, &globals);
                        }
                        PrimitiveBatch::PolychromeSprites { texture_id, sprites } => {
                            self.draw_poly_sprites(&mut render_pass, texture_id, sprites, &globals);
                        }
                        PrimitiveBatch::Surfaces(_surfaces) => {
                            // Video surfaces not supported on web
                        }
                        PrimitiveBatch::Paths(_) => unreachable!(),
                    }
                }
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        frame.present();
    }

    fn draw_paths_to_intermediate(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        paths: &[Path<ScaledPixels>],
        globals: &GlobalParams,
    ) {
        let Some(ref intermediate_view) = self.path_intermediate_texture_view else {
            return;
        };

        let mut vertices = Vec::new();
        for path in paths {
            let clipped_bounds: GpuBounds = path.clipped_bounds().into();
            let color: GpuBackground = path.color.clone().into();
            for vertex in &path.vertices {
                vertices.push(PathRasterizationVertex {
                    xy_position: [vertex.xy_position.x.0, vertex.xy_position.y.0],
                    st_position: [vertex.st_position.x, vertex.st_position.y],
                    color,
                    bounds: clipped_bounds,
                });
            }
        }

        if vertices.is_empty() {
            return;
        }

        let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("path rasterization globals buffer"),
            contents: bytemuck::bytes_of(globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_sprite_params = SpriteParams {
            gamma_ratios: [0.0; 4],
            grayscale_enhanced_contrast: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let sprite_params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite params buffer"),
            contents: bytemuck::bytes_of(&dummy_sprite_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_view = self.dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let vertices_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("path vertices buffer"),
            contents: slice_to_bytes(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("path rasterization bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sprite_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        {
            let (view, resolve_target) = if let Some(ref msaa_view) = self.path_intermediate_msaa_texture_view {
                (msaa_view, Some(intermediate_view as &wgpu::TextureView))
            } else {
                (intermediate_view, None)
            };

            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("path rasterization pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.pipelines.path_rasterization);
            render_pass.set_bind_group(0, &bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertices_buffer.slice(..));
            render_pass.draw(0..vertices.len() as u32, 0..1);
        }
    }

    fn draw_path_sprites(
        &self,
        render_pass: &mut wgpu::RenderPass,
        paths: &[Path<ScaledPixels>],
        intermediate_view: &wgpu::TextureView,
        globals: &GlobalParams,
    ) {
        if paths.is_empty() {
            return;
        }

        let first_path = &paths[0];
        let sprites: Vec<PathSprite> = if paths.last().map(|p| p.order) == Some(first_path.order) {
            paths
                .iter()
                .map(|path| PathSprite {
                    bounds: path.clipped_bounds().into(),
                })
                .collect()
        } else {
            let mut bounds = first_path.clipped_bounds();
            for path in paths.iter().skip(1) {
                bounds = bounds.union(&path.clipped_bounds());
            }
            vec![PathSprite {
                bounds: bounds.into(),
            }]
        };

        let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("path sprites globals buffer"),
            contents: bytemuck::bytes_of(globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_sprite_params = SpriteParams {
            gamma_ratios: [0.0; 4],
            grayscale_enhanced_contrast: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let sprite_params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite params buffer"),
            contents: bytemuck::bytes_of(&dummy_sprite_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let sprites_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("path sprites buffer"),
            contents: slice_to_bytes(&sprites),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("path sprites bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sprite_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(intermediate_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        render_pass.set_pipeline(&self.pipelines.paths);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, sprites_buffer.slice(..));
        render_pass.draw(0..4, 0..sprites.len() as u32);
    }

    fn draw_quads(
        &self,
        render_pass: &mut wgpu::RenderPass,
        quads: &[Quad],
        globals: &GlobalParams,
    ) {
        if quads.is_empty() {
            return;
        }

        let gpu_quads: Vec<GpuQuad> = quads.iter().map(GpuQuad::from).collect();

        let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals buffer"),
            contents: bytemuck::bytes_of(globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_sprite_params = SpriteParams {
            gamma_ratios: [0.0; 4],
            grayscale_enhanced_contrast: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let sprite_params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite params buffer"),
            contents: bytemuck::bytes_of(&dummy_sprite_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_view = self.dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let quads_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("quads buffer"),
            contents: slice_to_bytes(&gpu_quads),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("quads bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sprite_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        render_pass.set_pipeline(&self.pipelines.quads);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, quads_buffer.slice(..));
        render_pass.draw(0..4, 0..quads.len() as u32);
    }

    fn draw_shadows(
        &self,
        render_pass: &mut wgpu::RenderPass,
        shadows: &[Shadow],
        globals: &GlobalParams,
    ) {
        if shadows.is_empty() {
            return;
        }

        let gpu_shadows: Vec<GpuShadow> = shadows.iter().map(GpuShadow::from).collect();

        let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals buffer"),
            contents: bytemuck::bytes_of(globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_sprite_params = SpriteParams {
            gamma_ratios: [0.0; 4],
            grayscale_enhanced_contrast: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let sprite_params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite params buffer"),
            contents: bytemuck::bytes_of(&dummy_sprite_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_view = self.dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let shadows_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("shadows buffer"),
            contents: slice_to_bytes(&gpu_shadows),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shadows bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sprite_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        render_pass.set_pipeline(&self.pipelines.shadows);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, shadows_buffer.slice(..));
        render_pass.draw(0..4, 0..shadows.len() as u32);
    }

    fn draw_underlines(
        &self,
        render_pass: &mut wgpu::RenderPass,
        underlines: &[Underline],
        globals: &GlobalParams,
    ) {
        if underlines.is_empty() {
            return;
        }

        let gpu_underlines: Vec<GpuUnderline> = underlines.iter().map(GpuUnderline::from).collect();

        let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals buffer"),
            contents: bytemuck::bytes_of(globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_sprite_params = SpriteParams {
            gamma_ratios: [0.0; 4],
            grayscale_enhanced_contrast: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let sprite_params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite params buffer"),
            contents: bytemuck::bytes_of(&dummy_sprite_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_view = self.dummy_texture.create_view(&wgpu::TextureViewDescriptor::default());

        let underlines_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("underlines buffer"),
            contents: slice_to_bytes(&gpu_underlines),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("underlines bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sprite_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&dummy_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        render_pass.set_pipeline(&self.pipelines.underlines);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, underlines_buffer.slice(..));
        render_pass.draw(0..4, 0..underlines.len() as u32);
    }

    fn draw_mono_sprites(
        &self,
        render_pass: &mut wgpu::RenderPass,
        texture_id: crate::AtlasTextureId,
        sprites: &[MonochromeSprite],
        globals: &GlobalParams,
    ) {
        if sprites.is_empty() {
            return;
        }

        let gpu_sprites: Vec<GpuMonochromeSprite> = sprites.iter().map(GpuMonochromeSprite::from).collect();

        let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals buffer"),
            contents: bytemuck::bytes_of(globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let sprite_params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite params buffer"),
            contents: bytemuck::bytes_of(&self.rendering_parameters.sprite_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let texture_view = self.atlas.get_texture_view(texture_id);

        let sprites_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("mono sprites buffer"),
            contents: slice_to_bytes(&gpu_sprites),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("mono sprites bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sprite_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        render_pass.set_pipeline(&self.pipelines.mono_sprites);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, sprites_buffer.slice(..));
        render_pass.draw(0..4, 0..sprites.len() as u32);
    }

    fn draw_poly_sprites(
        &self,
        render_pass: &mut wgpu::RenderPass,
        texture_id: crate::AtlasTextureId,
        sprites: &[PolychromeSprite],
        globals: &GlobalParams,
    ) {
        if sprites.is_empty() {
            return;
        }

        let gpu_sprites: Vec<GpuPolychromeSprite> = sprites.iter().map(GpuPolychromeSprite::from).collect();

        let globals_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("globals buffer"),
            contents: bytemuck::bytes_of(globals),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let dummy_sprite_params = SpriteParams {
            gamma_ratios: [0.0; 4],
            grayscale_enhanced_contrast: 0.0,
            _pad0: 0.0,
            _pad1: 0.0,
            _pad2: 0.0,
        };
        let sprite_params_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sprite params buffer"),
            contents: bytemuck::bytes_of(&dummy_sprite_params),
            usage: wgpu::BufferUsages::UNIFORM,
        });

        let texture_view = self.atlas.get_texture_view(texture_id);

        let sprites_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("poly sprites buffer"),
            contents: slice_to_bytes(&gpu_sprites),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("poly sprites bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: globals_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: sprite_params_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&self.atlas_sampler),
                },
            ],
        });

        render_pass.set_pipeline(&self.pipelines.poly_sprites);
        render_pass.set_bind_group(0, &bind_group, &[]);
        render_pass.set_vertex_buffer(0, sprites_buffer.slice(..));
        render_pass.draw(0..4, 0..sprites.len() as u32);
    }
}

fn create_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("unified bind group layout"),
        entries: &[
            // binding 0: GlobalParams
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // binding 1: sprite_params (gamma_ratios + grayscale_enhanced_contrast, 32 bytes)
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // binding 2: t_sprite texture
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Texture {
                    sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    view_dimension: wgpu::TextureViewDimension::D2,
                    multisampled: false,
                },
                count: None,
            },
            // binding 3: s_sprite sampler
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                count: None,
            },
        ],
    })
}

fn create_pipelines(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    layout: &wgpu::BindGroupLayout,
    path_sample_count: u32,
) -> WgpuPipelines {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("gpui shaders"),
        source: wgpu::ShaderSource::Wgsl(include_str!("shaders.wgsl").into()),
    });

    let blend_state = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::SrcAlpha,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
    };

    // Premultiplied alpha blend state for path sprites, matching Metal's behavior.
    // Path rasterization outputs premultiplied colors (rgb * alpha), so we use
    // One instead of SrcAlpha for the source factor.
    let premultiplied_blend_state = wgpu::BlendState {
        color: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
            operation: wgpu::BlendOperation::Add,
        },
        alpha: wgpu::BlendComponent {
            src_factor: wgpu::BlendFactor::One,
            dst_factor: wgpu::BlendFactor::One,
            operation: wgpu::BlendOperation::Add,
        },
    };

    let color_target = wgpu::ColorTargetState {
        format,
        blend: Some(blend_state),
        write_mask: wgpu::ColorWrites::ALL,
    };

    let premultiplied_color_target = wgpu::ColorTargetState {
        format,
        blend: Some(premultiplied_blend_state),
        write_mask: wgpu::ColorWrites::ALL,
    };

    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("unified pipeline layout"),
        bind_group_layouts: &[layout],
        push_constant_ranges: &[],
    });

    let quad_layout = quad_vertex_buffer_layout();
    let quads = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("quads pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_quad"),
            buffers: &[quad_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_quad"),
            targets: &[Some(color_target.clone())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let shadow_layout = shadow_vertex_buffer_layout();
    let shadows = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("shadows pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_shadow"),
            buffers: &[shadow_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_shadow"),
            targets: &[Some(color_target.clone())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let underline_layout = underline_vertex_buffer_layout();
    let underlines = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("underlines pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_underline"),
            buffers: &[underline_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_underline"),
            targets: &[Some(color_target.clone())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let mono_sprite_layout = mono_sprite_vertex_buffer_layout();
    let mono_sprites = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("mono sprites pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_mono_sprite"),
            buffers: &[mono_sprite_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_mono_sprite"),
            targets: &[Some(color_target.clone())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let poly_sprite_layout = poly_sprite_vertex_buffer_layout();
    let poly_sprites = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("poly sprites pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_poly_sprite"),
            buffers: &[poly_sprite_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_poly_sprite"),
            targets: &[Some(color_target.clone())],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    let path_rasterization_layout = path_rasterization_vertex_buffer_layout();
    let path_rasterization = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("path rasterization pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_path_rasterization"),
            buffers: &[path_rasterization_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_path_rasterization"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: path_sample_count,
            ..Default::default()
        },
        multiview: None,
        cache: None,
    });

    let path_sprite_layout = path_sprite_vertex_buffer_layout();
    let paths = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("paths pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_path"),
            buffers: &[path_sprite_layout],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_path"),
            // Path sprites contain premultiplied alpha colors from the path rasterization
            // pass, so we need to use One instead of SrcAlpha for the source factor.
            targets: &[Some(premultiplied_color_target)],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleStrip,
            ..Default::default()
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview: None,
        cache: None,
    });

    WgpuPipelines {
        quads,
        shadows,
        underlines,
        mono_sprites,
        poly_sprites,
        path_rasterization,
        paths,
    }
}
