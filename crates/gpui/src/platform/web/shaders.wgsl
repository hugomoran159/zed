/* Functions useful for debugging:

// A heat map color for debugging (blue -> cyan -> green -> yellow -> red).
fn heat_map_color(value: f32, minValue: f32, maxValue: f32, position: vec2<f32>) -> vec4<f32> {
    // Normalize value to 0-1 range
    let t = clamp((value - minValue) / (maxValue - minValue), 0.0, 1.0);

    // Heat map color calculation
    let r = t * t;
    let g = 4.0 * t * (1.0 - t);
    let b = (1.0 - t) * (1.0 - t);
    let heat_color = vec3<f32>(r, g, b);

    // Create a checkerboard pattern (black and white)
    let sum = floor(position.x / 3) + floor(position.y / 3);
    let is_odd = fract(sum * 0.5); // 0.0 for even, 0.5 for odd
    let checker_value = is_odd * 2.0; // 0.0 for even, 1.0 for odd
    let checker_color = vec3<f32>(checker_value);

    // Determine if value is in range (1.0 if in range, 0.0 if out of range)
    let in_range = step(minValue, value) * step(value, maxValue);

    // Mix checkerboard and heat map based on whether value is in range
    let final_color = mix(checker_color, heat_color, in_range);

    return vec4<f32>(final_color, 1.0);
}

*/

// Contrast and gamma correction adapted from https://github.com/microsoft/terminal/blob/1283c0f5b99a2961673249fa77c6b986efb5086c/src/renderer/atlas/dwrite.hlsl
// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
fn color_brightness(color: vec3<f32>) -> f32 {
    // REC. 601 luminance coefficients for perceived brightness
    return dot(color, vec3<f32>(0.30, 0.59, 0.11));
}

fn light_on_dark_contrast(enhancedContrast: f32, color: vec3<f32>) -> f32 {
    let brightness = color_brightness(color);
    let multiplier = saturate(4.0 * (0.75 - brightness));
    return enhancedContrast * multiplier;
}

fn enhance_contrast(alpha: f32, k: f32) -> f32 {
    return alpha * (k + 1.0) / (alpha * k + 1.0);
}

fn apply_alpha_correction(a: f32, b: f32, g: vec4<f32>) -> f32 {
    let brightness_adjustment = g.x * b + g.y;
    let correction = brightness_adjustment * a + (g.z * b + g.w);
    return a + a * (1.0 - a) * correction;
}

fn apply_contrast_and_gamma_correction(sample: f32, color: vec3<f32>, enhanced_contrast_factor: f32, gamma_ratios_val: vec4<f32>) -> f32 {
    let enhanced_contrast = light_on_dark_contrast(enhanced_contrast_factor, color);
    let brightness = color_brightness(color);

    let contrasted = enhance_contrast(sample, enhanced_contrast);
    return apply_alpha_correction(contrasted, brightness, gamma_ratios_val);
}

// Global uniforms - matching Blade's structure
struct GlobalParams {
    viewport_size: vec2<f32>,
    premultiplied_alpha: u32,
    pad: u32,
}

// For quads, shadows, underlines - just globals and instance data
@group(0) @binding(0) var<uniform> globals: GlobalParams;

// Sprite rendering parameters - packed for WebGL 16-byte alignment
struct SpriteParams {
    gamma_ratios: vec4<f32>,
    grayscale_enhanced_contrast: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

// For sprites - additional parameters
@group(0) @binding(1) var<uniform> sprite_params: SpriteParams;
@group(0) @binding(2) var t_sprite: texture_2d<f32>;
@group(0) @binding(3) var s_sprite: sampler;

const M_PI_F: f32 = 3.1415926;
const GRAYSCALE_FACTORS: vec3<f32> = vec3<f32>(0.2126, 0.7152, 0.0722);
const UNDERLINE_WAVE_FREQUENCY: f32 = 2.0;
const UNDERLINE_WAVE_HEIGHT_RATIO: f32 = 0.8;

struct Bounds {
    origin: vec2<f32>,
    size: vec2<f32>,
}

struct Corners {
    top_left: f32,
    top_right: f32,
    bottom_right: f32,
    bottom_left: f32,
}

struct Edges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

struct Hsla {
    h: f32,
    s: f32,
    l: f32,
    a: f32,
}

struct LinearColorStop {
    color: Hsla,
    percentage: f32,
}

struct Background {
    // 0u is Solid
    // 1u is LinearGradient
    // 2u is PatternSlash
    tag: u32,
    // 0u is sRGB linear color
    // 1u is Oklab color
    color_space: u32,
    solid: Hsla,
    gradient_angle_or_pattern_height: f32,
    colors: array<LinearColorStop, 2>,
    pad: u32,
}

struct AtlasTextureId {
    index: u32,
    kind: u32,
}

struct AtlasBounds {
    origin: vec2<i32>,
    size: vec2<i32>,
}

struct AtlasTile {
    texture_id: AtlasTextureId,
    tile_id: u32,
    padding: u32,
    bounds: AtlasBounds,
}

struct TransformationMatrix {
    rotation_scale: mat2x2<f32>,
    translation: vec2<f32>,
}

fn to_device_position_impl(position: vec2<f32>) -> vec4<f32> {
    let device_position = position / globals.viewport_size * vec2<f32>(2.0, -2.0) + vec2<f32>(-1.0, 1.0);
    return vec4<f32>(device_position, 0.0, 1.0);
}

fn to_device_position(unit_vertex: vec2<f32>, bounds: Bounds) -> vec4<f32> {
    let position = unit_vertex * vec2<f32>(bounds.size) + bounds.origin;
    return to_device_position_impl(position);
}

fn to_device_position_transformed(unit_vertex: vec2<f32>, bounds: Bounds, transform: TransformationMatrix) -> vec4<f32> {
    let position = unit_vertex * vec2<f32>(bounds.size) + bounds.origin;
    //Note: Rust side stores it as row-major, so transposing here
    let transformed = transpose(transform.rotation_scale) * position + transform.translation;
    return to_device_position_impl(transformed);
}

fn to_tile_position(unit_vertex: vec2<f32>, tile_bounds_origin: vec2<i32>, tile_bounds_size: vec2<i32>) -> vec2<f32> {
  let atlas_size = vec2<f32>(textureDimensions(t_sprite, 0));
  return (vec2<f32>(tile_bounds_origin) + unit_vertex * vec2<f32>(tile_bounds_size)) / atlas_size;
}

fn distance_from_clip_rect_impl(position: vec2<f32>, clip_bounds: Bounds) -> vec4<f32> {
    let tl = position - clip_bounds.origin;
    let br = clip_bounds.origin + clip_bounds.size - position;
    return vec4<f32>(tl.x, br.x, tl.y, br.y);
}

fn distance_from_clip_rect(unit_vertex: vec2<f32>, bounds: Bounds, clip_bounds: Bounds) -> vec4<f32> {
    let position = unit_vertex * vec2<f32>(bounds.size) + bounds.origin;
    return distance_from_clip_rect_impl(position, clip_bounds);
}

fn distance_from_clip_rect_transformed(unit_vertex: vec2<f32>, bounds: Bounds, clip_bounds: Bounds, transform: TransformationMatrix) -> vec4<f32> {
    let position = unit_vertex * vec2<f32>(bounds.size) + bounds.origin;
    let transformed = transpose(transform.rotation_scale) * position + transform.translation;
    return distance_from_clip_rect_impl(transformed, clip_bounds);
}

// Use simple gamma 2.2 approximation to match Metal shader behavior
fn srgb_to_linear(srgb: vec3<f32>) -> vec3<f32> {
    return pow(srgb, vec3<f32>(2.2));
}

fn linear_to_srgb(linear: vec3<f32>) -> vec3<f32> {
    return pow(linear, vec3<f32>(1.0 / 2.2));
}

/// Convert a linear color to sRGBA space.
fn linear_to_srgba(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(linear_to_srgb(color.rgb), color.a);
}

/// Convert a sRGBA color to linear space.
fn srgba_to_linear(color: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(srgb_to_linear(color.rgb), color.a);
}

/// Hsla to linear RGBA conversion.
fn hsla_to_rgba(hsla: Hsla) -> vec4<f32> {
    let h = hsla.h * 6.0; // Now, it's an angle but scaled in [0, 6) range
    let s = hsla.s;
    let l = hsla.l;
    let a = hsla.a;

    let c = (1.0 - abs(2.0 * l - 1.0)) * s;
    let x = c * (1.0 - abs(h % 2.0 - 1.0));
    let m = l - c / 2.0;
    var color = vec3<f32>(m);

    if (h >= 0.0 && h < 1.0) {
        color.r += c;
        color.g += x;
    } else if (h >= 1.0 && h < 2.0) {
        color.r += x;
        color.g += c;
    } else if (h >= 2.0 && h < 3.0) {
        color.g += c;
        color.b += x;
    } else if (h >= 3.0 && h < 4.0) {
        color.g += x;
        color.b += c;
    } else if (h >= 4.0 && h < 5.0) {
        color.r += x;
        color.b += c;
    } else {
        color.r += c;
        color.b += x;
    }

    return vec4<f32>(color, a);
}

/// Convert a linear sRGB to Oklab space.
/// Reference: https://bottosson.github.io/posts/oklab/#converting-from-linear-srgb-to-oklab
fn linear_srgb_to_oklab(color: vec4<f32>) -> vec4<f32> {
	let l = 0.4122214708 * color.r + 0.5363325363 * color.g + 0.0514459929 * color.b;
	let m = 0.2119034982 * color.r + 0.6806995451 * color.g + 0.1073969566 * color.b;
	let s = 0.0883024619 * color.r + 0.2817188376 * color.g + 0.6299787005 * color.b;

	let l_ = pow(l, 1.0 / 3.0);
	let m_ = pow(m, 1.0 / 3.0);
	let s_ = pow(s, 1.0 / 3.0);

	return vec4<f32>(
		0.2104542553 * l_ + 0.7936177850 * m_ - 0.0040720468 * s_,
		1.9779984951 * l_ - 2.4285922050 * m_ + 0.4505937099 * s_,
		0.0259040371 * l_ + 0.7827717662 * m_ - 0.8086757660 * s_,
		color.a
	);
}

/// Convert an Oklab color to linear sRGB space.
fn oklab_to_linear_srgb(color: vec4<f32>) -> vec4<f32> {
	let l_ = color.r + 0.3963377774 * color.g + 0.2158037573 * color.b;
	let m_ = color.r - 0.1055613458 * color.g - 0.0638541728 * color.b;
	let s_ = color.r - 0.0894841775 * color.g - 1.2914855480 * color.b;

	let l = l_ * l_ * l_;
	let m = m_ * m_ * m_;
	let s = s_ * s_ * s_;

	return vec4<f32>(
		4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s,
		-1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s,
		-0.0041960863 * l - 0.7034186147 * m + 1.7076147010 * s,
		color.a
	);
}

/// Convert an Oklab color to non-linear sRGB space (matches Metal's oklab_to_srgb)
fn oklab_to_srgb(color: vec4<f32>) -> vec4<f32> {
    let linear = oklab_to_linear_srgb(color);
    return vec4<f32>(linear_to_srgb(linear.rgb), linear.a);
}

fn over(below: vec4<f32>, above: vec4<f32>) -> vec4<f32> {
    let alpha = above.a + below.a * (1.0 - above.a);
    let color = (above.rgb * above.a + below.rgb * below.a * (1.0 - above.a)) / alpha;
    return vec4<f32>(color, alpha);
}

// A standard gaussian function, used for weighting samples
fn gaussian(x: f32, sigma: f32) -> f32{
    return exp(-(x * x) / (2.0 * sigma * sigma)) / (sqrt(2.0 * M_PI_F) * sigma);
}

// This approximates the error function, needed for the gaussian integral
fn erf(v: vec2<f32>) -> vec2<f32> {
    let s = sign(v);
    let a = abs(v);
    let r1 = 1.0 + (0.278393 + (0.230389 + (0.000972 + 0.078108 * a) * a) * a) * a;
    let r2 = r1 * r1;
    return s - s / (r2 * r2);
}

fn blur_along_x(x: f32, y: f32, sigma: f32, corner: f32, half_size: vec2<f32>) -> f32 {
  let delta = min(half_size.y - corner - abs(y), 0.0);
  let curved = half_size.x - corner + sqrt(max(0.0, corner * corner - delta * delta));
  let integral = 0.5 + 0.5 * erf((x + vec2<f32>(-curved, curved)) * (sqrt(0.5) / sigma));
  return integral.y - integral.x;
}

// Selects corner radius based on quadrant.
fn pick_corner_radius(center_to_point: vec2<f32>, radii: Corners) -> f32 {
    if (center_to_point.x < 0.0) {
        if (center_to_point.y < 0.0) {
            return radii.top_left;
        } else {
            return radii.bottom_left;
        }
    } else {
        if (center_to_point.y < 0.0) {
            return radii.top_right;
        } else {
            return radii.bottom_right;
        }
    }
}

// Signed distance of the point to the quad's border - positive outside the
// border, and negative inside.
//
// See comments on similar code using `quad_sdf_impl` in `fs_quad` for
// explanation.
fn quad_sdf(point: vec2<f32>, bounds: Bounds, corner_radii: Corners) -> f32 {
    let half_size = bounds.size / 2.0;
    let center = bounds.origin + half_size;
    let center_to_point = point - center;
    let corner_radius = pick_corner_radius(center_to_point, corner_radii);
    let corner_to_point = abs(center_to_point) - half_size;
    let corner_center_to_point = corner_to_point + corner_radius;
    return quad_sdf_impl(corner_center_to_point, corner_radius);
}

fn quad_sdf_impl(corner_center_to_point: vec2<f32>, corner_radius: f32) -> f32 {
    if (corner_radius == 0.0) {
        // Fast path for unrounded corners.
        return max(corner_center_to_point.x, corner_center_to_point.y);
    } else {
        // Signed distance of the point from a quad that is inset by corner_radius.
        // It is negative inside this quad, and positive outside.
        let signed_distance_to_inset_quad =
            // 0 inside the inset quad, and positive outside.
            length(max(vec2<f32>(0.0), corner_center_to_point)) +
            // 0 outside the inset quad, and negative inside.
            min(0.0, max(corner_center_to_point.x, corner_center_to_point.y));

        return signed_distance_to_inset_quad - corner_radius;
    }
}

// Abstract away the final color transformation based on the
// target alpha compositing mode.
fn blend_color(color: vec4<f32>, alpha_factor: f32) -> vec4<f32> {
    let alpha = color.a * alpha_factor;
    let multiplier = select(1.0, alpha, globals.premultiplied_alpha != 0u);
    return vec4<f32>(color.rgb * multiplier, alpha);
}


struct GradientColor {
    solid: vec4<f32>,
    color0: vec4<f32>,
    color1: vec4<f32>,
}

fn prepare_gradient_color(tag: u32, color_space: u32,
    solid: Hsla, colors: array<LinearColorStop, 2>) -> GradientColor {
    var result = GradientColor();

    if (tag == 0u || tag == 2u) {
        result.solid = hsla_to_rgba(solid);
    } else if (tag == 1u) {
        result.color0 = hsla_to_rgba(colors[0].color);
        result.color1 = hsla_to_rgba(colors[1].color);

        // Prepare color space in vertex shader for performance
        // Match Metal behavior: only convert for Oklab color space
        if (color_space == 1u) {
            // Oklab - convert sRGB to Oklab (Metal uses srgb_to_oklab which
            // first converts to linear then to Oklab)
            result.color0 = linear_srgb_to_oklab(srgba_to_linear(result.color0));
            result.color1 = linear_srgb_to_oklab(srgba_to_linear(result.color1));
        }
        // For sRGB (color_space == 0u), keep colors as-is like Metal does
    }

    return result;
}

fn gradient_color_from_prepared(tag: u32, color_space: u32, gradient_angle_or_pattern_height: f32,
    stop0_percentage: f32, stop1_percentage: f32,
    position: vec2<f32>, bounds: Bounds,
    solid_color: vec4<f32>, color0: vec4<f32>, color1: vec4<f32>) -> vec4<f32> {
    var background_color = vec4<f32>(0.0);

    switch (tag) {
        default: {
            return solid_color;
        }
        case 1u: {
            // Linear gradient background.
            // -90 degrees to match the CSS gradient angle.
            let angle = gradient_angle_or_pattern_height;
            let radians = (angle % 360.0 - 90.0) * M_PI_F / 180.0;
            var direction = vec2<f32>(cos(radians), sin(radians));

            // Expand the short side to be the same as the long side
            if (bounds.size.x > bounds.size.y) {
                direction.y *= bounds.size.y / bounds.size.x;
            } else {
                direction.x *= bounds.size.x / bounds.size.y;
            }

            // Get the t value for the linear gradient with the color stop percentages.
            let half_size = bounds.size / 2.0;
            let center = bounds.origin + half_size;
            let center_to_point = position - center;
            var t = dot(center_to_point, direction) / length(direction);
            // Check the direct to determine the use x or y
            if (abs(direction.x) > abs(direction.y)) {
                t = (t + half_size.x) / bounds.size.x;
            } else {
                t = (t + half_size.y) / bounds.size.y;
            }

            // Adjust t based on the stop percentages
            t = (t - stop0_percentage) / (stop1_percentage - stop0_percentage);
            t = clamp(t, 0.0, 1.0);

            switch (color_space) {
                default: {
                    // sRGB: mix directly like Metal does
                    background_color = mix(color0, color1, t);
                }
                case 1u: {
                    // Oklab: mix in Oklab space then convert back to sRGB
                    let oklab_color = mix(color0, color1, t);
                    background_color = oklab_to_srgb(oklab_color);
                }
            }
        }
        case 2u: {
            let pattern_width = (gradient_angle_or_pattern_height / 65535.0f) / 255.0f;
            let pattern_interval = (gradient_angle_or_pattern_height % 65535.0f) / 255.0f;
            let pattern_height = pattern_width + pattern_interval;
            let stripe_angle = M_PI_F / 4.0;
            let pattern_period = pattern_height * sin(stripe_angle);
            let rotation = mat2x2<f32>(
                cos(stripe_angle), -sin(stripe_angle),
                sin(stripe_angle), cos(stripe_angle)
            );
            let relative_position = position - bounds.origin;
            let rotated_point = rotation * relative_position;
            let pattern = rotated_point.x % pattern_period;
            let distance = min(pattern, pattern_period - pattern) - pattern_period * (pattern_width / pattern_height) /  2.0f;
            background_color = solid_color;
            background_color.a *= saturate(0.5 - distance);
        }
    }

    return background_color;
}

// This approximates distance to the nearest point to a quarter ellipse in a way
// that is sufficient for anti-aliasing when the ellipse is not very eccentric.
// Negative on the outside and positive on the inside.
fn quarter_ellipse_sdf(point: vec2<f32>, radii: vec2<f32>) -> f32 {
    // Scale the space to treat the ellipse like a unit circle.
    let circle_vec = point / radii;
    let unit_circle_sdf = length(circle_vec) - 1.0;
    return unit_circle_sdf * (radii.x + radii.y) * -0.5;
}

// Modulus that has the same sign as `a`.
fn fmod(a: f32, b: f32) -> f32 {
    return a - b * trunc(a / b);
}

// --- quads --- //

struct QuadVertexInput {
    @location(0) bounds_origin: vec2<f32>,
    @location(1) bounds_size: vec2<f32>,
    @location(2) content_mask_origin: vec2<f32>,
    @location(3) content_mask_size: vec2<f32>,
    @location(4) background_tag_colorspace: vec2<u32>,
    @location(5) background_solid: vec4<f32>,
    @location(6) background_grad1: vec4<f32>,  // gradient_angle + colors[0].color.hsl
    @location(7) background_grad2: vec4<f32>,  // colors[0].percentage + colors[1].color.hsl
    @location(8) background_grad3: vec4<f32>,  // colors[1].color.a + colors[1].percentage + pad
    @location(9) border_color: vec4<f32>,
    @location(10) corner_radii: vec4<f32>,
    @location(11) border_widths: vec4<f32>,
}

struct QuadVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) border_color: vec4<f32>,
    @location(1) clip_distances: vec4<f32>,
    @location(2) @interpolate(flat) background_solid: vec4<f32>,
    @location(3) @interpolate(flat) bounds: vec4<f32>,  // origin.xy, size.xy packed
    @location(4) @interpolate(flat) corner_radii: vec4<f32>,
    @location(5) @interpolate(flat) border_widths: vec4<f32>,
    @location(6) @interpolate(flat) background_tag_colorspace: vec2<u32>,
    // Gradient data: color0 (RGBA)
    @location(7) @interpolate(flat) gradient_color0: vec4<f32>,
    // Gradient data: color1 (RGBA), plus angle/stops packed in w component area
    @location(8) @interpolate(flat) gradient_color1: vec4<f32>,
    // Gradient params: [angle, stop0, stop1, unused]
    @location(9) @interpolate(flat) gradient_params: vec4<f32>,
}

@vertex
fn vs_quad(@builtin(vertex_index) vertex_id: u32, input: QuadVertexInput) -> QuadVarying {
    let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));

    let bounds = Bounds(input.bounds_origin, input.bounds_size);
    let content_mask = Bounds(input.content_mask_origin, input.content_mask_size);

    // Parse background solid color
    let solid = Hsla(input.background_solid.x, input.background_solid.y, input.background_solid.z, input.background_solid.w);

    // Parse gradient data from vertex input
    // Layout: grad1 = [angle, h0, s0, l0], grad2 = [a0, stop0, h1, s1], grad3 = [l1, a1, stop1, pad]
    let gradient_angle = input.background_grad1.x;
    let color0_hsla = Hsla(input.background_grad1.y, input.background_grad1.z, input.background_grad1.w, input.background_grad2.x);
    let stop0_percentage = input.background_grad2.y;
    let color1_hsla = Hsla(input.background_grad2.z, input.background_grad2.w, input.background_grad3.x, input.background_grad3.y);
    let stop1_percentage = input.background_grad3.z;

    var out = QuadVarying();
    out.position = to_device_position(unit_vertex, bounds);
    out.background_solid = hsla_to_rgba(solid);
    out.border_color = hsla_to_rgba(Hsla(input.border_color.x, input.border_color.y, input.border_color.z, input.border_color.w));
    out.background_tag_colorspace = input.background_tag_colorspace;
    out.clip_distances = distance_from_clip_rect(unit_vertex, bounds, content_mask);
    out.bounds = vec4<f32>(input.bounds_origin, input.bounds_size);
    out.corner_radii = input.corner_radii;
    out.border_widths = input.border_widths;

    // Pass gradient colors (pre-converted to RGBA for efficiency)
    out.gradient_color0 = hsla_to_rgba(color0_hsla);
    out.gradient_color1 = hsla_to_rgba(color1_hsla);
    out.gradient_params = vec4<f32>(gradient_angle, stop0_percentage, stop1_percentage, 0.0);

    return out;
}

// Compute the background color for a quad, supporting solid colors and linear gradients
fn compute_quad_background_color(
    position: vec2<f32>,
    bounds: Bounds,
    background_tag: u32,
    color_space: u32,
    solid_color: vec4<f32>,
    gradient_color0: vec4<f32>,
    gradient_color1: vec4<f32>,
    gradient_params: vec4<f32>,  // [angle, stop0, stop1, unused]
) -> vec4<f32> {
    // background_tag: 0 = Solid, 1 = LinearGradient, 2 = PatternSlash
    if (background_tag == 0u) {
        return solid_color;
    } else if (background_tag == 1u) {
        // Linear gradient
        let angle = gradient_params.x;
        let stop0 = gradient_params.y;
        let stop1 = gradient_params.z;

        // Compute normalized position within bounds
        let normalized_pos = (position - bounds.origin) / bounds.size;

        // Convert angle to radians and adjust for CSS gradient convention
        // -90 degrees to match the CSS gradient angle (0 = bottom to top)
        let angle_rad = radians(angle - 90.0);
        let dir = vec2<f32>(cos(angle_rad), sin(angle_rad));

        // Project position onto gradient direction
        // Center the gradient at (0.5, 0.5) and project
        let centered = normalized_pos - vec2<f32>(0.5, 0.5);
        let projected = dot(centered, dir) + 0.5;

        // Compute t value with color stops
        let t = clamp((projected - stop0) / (stop1 - stop0), 0.0, 1.0);

        // Interpolate based on color space
        if (color_space == 1u) {
            // Oklab interpolation
            let lab0 = linear_srgb_to_oklab(gradient_color0);
            let lab1 = linear_srgb_to_oklab(gradient_color1);
            let lab_mixed = mix(lab0, lab1, t);
            return oklab_to_linear_srgb(lab_mixed);
        } else {
            // sRGB interpolation (default)
            return mix(gradient_color0, gradient_color1, t);
        }
    } else if (background_tag == 2u) {
        // PatternSlash
        // gradient_params.x contains the encoded pattern_height value
        let gradient_angle_or_pattern_height = gradient_params.x;
        let pattern_width = (gradient_angle_or_pattern_height / 65535.0f) / 255.0f;
        let pattern_interval = (gradient_angle_or_pattern_height % 65535.0f) / 255.0f;
        let pattern_height = pattern_width + pattern_interval;
        let stripe_angle = M_PI_F / 4.0;
        let pattern_period = pattern_height * sin(stripe_angle);
        let rotation = mat2x2<f32>(
            cos(stripe_angle), -sin(stripe_angle),
            sin(stripe_angle), cos(stripe_angle)
        );
        let relative_position = position - bounds.origin;
        let rotated_point = rotation * relative_position;
        let pattern = rotated_point.x % pattern_period;
        let distance = min(pattern, pattern_period - pattern) - pattern_period * (pattern_width / pattern_height) / 2.0f;
        var background_color = solid_color;
        background_color.a *= saturate(0.5 - distance);
        return background_color;
    } else {
        // Unknown tag - fall back to solid
        return solid_color;
    }
}

@fragment
fn fs_quad(input: QuadVarying) -> @location(0) vec4<f32> {
    // Alpha clip first, since we don't have `clip_distance`.
    if (any(input.clip_distances < vec4<f32>(0.0))) {
        return vec4<f32>(0.0);
    }

    let bounds = Bounds(input.bounds.xy, input.bounds.zw);
    let corner_radii = Corners(input.corner_radii.x, input.corner_radii.y, input.corner_radii.z, input.corner_radii.w);
    let border_widths = Edges(input.border_widths.x, input.border_widths.y, input.border_widths.z, input.border_widths.w);

    // Compute background color (supports solid and gradient)
    let background_color = compute_quad_background_color(
        input.position.xy,
        bounds,
        input.background_tag_colorspace.x,
        input.background_tag_colorspace.y,
        input.background_solid,
        input.gradient_color0,
        input.gradient_color1,
        input.gradient_params
    );

    let unrounded = corner_radii.top_left == 0.0 &&
        corner_radii.bottom_left == 0.0 &&
        corner_radii.top_right == 0.0 &&
        corner_radii.bottom_right == 0.0;

    // Fast path when the quad is not rounded and doesn't have any border
    if (border_widths.top == 0.0 &&
            border_widths.left == 0.0 &&
            border_widths.right == 0.0 &&
            border_widths.bottom == 0.0 &&
            unrounded) {
        return blend_color(background_color, 1.0);
    }

    let size = bounds.size;
    let half_size = size / 2.0;
    let point = input.position.xy - bounds.origin;
    let center_to_point = point - half_size;

    // Signed distance field threshold for inclusion of pixels. 0.5 is the
    // minimum distance between the center of the pixel and the edge.
    let antialias_threshold = 0.5;

    // Radius of the nearest corner
    let corner_radius = pick_corner_radius(center_to_point, corner_radii);

    // Width of the nearest borders
    let border = vec2<f32>(
        select(
            border_widths.right,
            border_widths.left,
            center_to_point.x < 0.0),
        select(
            border_widths.bottom,
            border_widths.top,
            center_to_point.y < 0.0));

    // 0-width borders are reduced so that `inner_sdf >= antialias_threshold`.
    // The purpose of this is to not draw antialiasing pixels in this case.
    let reduced_border =
        vec2<f32>(select(border.x, -antialias_threshold, border.x == 0.0),
                  select(border.y, -antialias_threshold, border.y == 0.0));

    // Vector from the corner of the quad bounds to the point, after mirroring
    // the point into the bottom right quadrant. Both components are <= 0.
    let corner_to_point = abs(center_to_point) - half_size;

    // Vector from the point to the center of the rounded corner's circle, also
    // mirrored into bottom right quadrant.
    let corner_center_to_point = corner_to_point + corner_radius;

    // Whether the nearest point on the border is rounded
    let is_near_rounded_corner =
            corner_center_to_point.x >= 0 &&
            corner_center_to_point.y >= 0;

    // Vector from straight border inner corner to point.
    let straight_border_inner_corner_to_point = corner_to_point + reduced_border;

    // Whether the point is beyond the inner edge of the straight border.
    let is_beyond_inner_straight_border =
            straight_border_inner_corner_to_point.x > 0 ||
            straight_border_inner_corner_to_point.y > 0;

    // Whether the point is far enough inside the quad, such that the pixels are
    // not affected by the straight border.
    let is_within_inner_straight_border =
        straight_border_inner_corner_to_point.x < -antialias_threshold &&
        straight_border_inner_corner_to_point.y < -antialias_threshold;

    // Fast path for points that must be part of the background.
    if (is_within_inner_straight_border && !is_near_rounded_corner) {
        return blend_color(background_color, 1.0);
    }

    // Signed distance of the point to the outside edge of the quad's border.
    let outer_sdf = quad_sdf_impl(corner_center_to_point, corner_radius);

    // Approximate signed distance of the point to the inside edge of the quad's
    // border. It is negative outside this edge (within the border), and
    // positive inside.
    var inner_sdf = 0.0;
    if (corner_center_to_point.x <= 0 || corner_center_to_point.y <= 0) {
        // Fast paths for straight borders.
        inner_sdf = -max(straight_border_inner_corner_to_point.x,
                         straight_border_inner_corner_to_point.y);
    } else if (is_beyond_inner_straight_border) {
        // Fast path for points that must be outside the inner edge.
        inner_sdf = -1.0;
    } else if (reduced_border.x == reduced_border.y) {
        // Fast path for circular inner edge.
        inner_sdf = -(outer_sdf + reduced_border.x);
    } else {
        let ellipse_radii = max(vec2<f32>(0.0), corner_radius - reduced_border);
        inner_sdf = quarter_ellipse_sdf(corner_center_to_point, ellipse_radii);
    }

    // Negative when inside the border
    let border_sdf = max(inner_sdf, outer_sdf);

    var color = background_color;
    if (border_sdf < antialias_threshold) {
        var border_color = input.border_color;

        // Blend the border on top of the background and then linearly interpolate
        // between the two as we slide inside the background.
        let blended_border = over(background_color, border_color);
        color = mix(background_color, blended_border,
                    saturate(antialias_threshold - inner_sdf));
    }

    return blend_color(color, saturate(antialias_threshold - outer_sdf));
}

// --- shadows --- //

struct ShadowVertexInput {
    @location(0) blur_radius_pad: vec4<f32>,
    @location(1) bounds_origin: vec2<f32>,
    @location(2) bounds_size: vec2<f32>,
    @location(3) corner_radii: vec4<f32>,
    @location(4) content_mask_origin: vec2<f32>,
    @location(5) content_mask_size: vec2<f32>,
    @location(6) color: vec4<f32>,
}

struct ShadowVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    @location(1) @interpolate(flat) blur_radius: f32,
    @location(2) clip_distances: vec4<f32>,
    @location(3) @interpolate(flat) bounds_origin: vec2<f32>,
    @location(4) @interpolate(flat) bounds_size: vec2<f32>,
    @location(5) @interpolate(flat) corner_radii: vec4<f32>,
}

@vertex
fn vs_shadow(@builtin(vertex_index) vertex_id: u32, input: ShadowVertexInput) -> ShadowVarying {
    let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));

    let blur_radius = input.blur_radius_pad.x;
    var bounds = Bounds(input.bounds_origin, input.bounds_size);
    let content_mask = Bounds(input.content_mask_origin, input.content_mask_size);

    let margin = 3.0 * blur_radius;
    bounds.origin -= vec2<f32>(margin);
    bounds.size += 2.0 * vec2<f32>(margin);

    var out = ShadowVarying();
    out.position = to_device_position(unit_vertex, bounds);
    out.color = hsla_to_rgba(Hsla(input.color.x, input.color.y, input.color.z, input.color.w));
    out.blur_radius = blur_radius;
    out.clip_distances = distance_from_clip_rect(unit_vertex, bounds, content_mask);
    out.bounds_origin = input.bounds_origin;
    out.bounds_size = input.bounds_size;
    out.corner_radii = input.corner_radii;
    return out;
}

@fragment
fn fs_shadow(input: ShadowVarying) -> @location(0) vec4<f32> {
    if (any(input.clip_distances < vec4<f32>(0.0))) {
        return vec4<f32>(0.0);
    }

    let bounds = Bounds(input.bounds_origin, input.bounds_size);
    let corner_radii = Corners(input.corner_radii.x, input.corner_radii.y, input.corner_radii.z, input.corner_radii.w);

    let half_size = bounds.size / 2.0;
    let center = bounds.origin + half_size;
    let center_to_point = input.position.xy - center;

    let corner_radius = pick_corner_radius(center_to_point, corner_radii);

    var alpha: f32;
    if (input.blur_radius == 0.0) {
        // Fast path for non-blurred shadows (matches Metal optimization)
        let distance = quad_sdf(input.position.xy, bounds, corner_radii);
        alpha = saturate(0.5 - distance);
    } else {
        let low = center_to_point.y - half_size.y;
        let high = center_to_point.y + half_size.y;
        let start = clamp(-3.0 * input.blur_radius, low, high);
        let end = clamp(3.0 * input.blur_radius, low, high);

        let step = (end - start) / 4.0;
        var y = start + step * 0.5;
        alpha = 0.0;
        for (var i = 0; i < 4; i += 1) {
            let blur = blur_along_x(center_to_point.x, center_to_point.y - y,
                input.blur_radius, corner_radius, half_size);
            alpha += blur * gaussian(y, input.blur_radius) * step;
            y += step;
        }
    }

    return blend_color(input.color, alpha);
}

// --- underlines --- //

struct UnderlineVertexInput {
    @location(0) bounds_origin: vec2<f32>,
    @location(1) bounds_size: vec2<f32>,
    @location(2) content_mask_origin: vec2<f32>,
    @location(3) content_mask_size: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) thickness_wavy_pad: vec4<f32>,
}

struct UnderlineVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) color: vec4<f32>,
    @location(1) @interpolate(flat) thickness: f32,
    @location(2) @interpolate(flat) wavy: u32,
    @location(3) clip_distances: vec4<f32>,
    @location(4) @interpolate(flat) bounds_origin: vec2<f32>,
    @location(5) @interpolate(flat) bounds_size: vec2<f32>,
}

@vertex
fn vs_underline(@builtin(vertex_index) vertex_id: u32, input: UnderlineVertexInput) -> UnderlineVarying {
    let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));
    let bounds = Bounds(input.bounds_origin, input.bounds_size);
    let content_mask = Bounds(input.content_mask_origin, input.content_mask_size);

    var out = UnderlineVarying();
    out.position = to_device_position(unit_vertex, bounds);
    out.color = hsla_to_rgba(Hsla(input.color.x, input.color.y, input.color.z, input.color.w));
    out.thickness = input.thickness_wavy_pad.x;
    out.wavy = u32(input.thickness_wavy_pad.y);
    out.clip_distances = distance_from_clip_rect(unit_vertex, bounds, content_mask);
    out.bounds_origin = input.bounds_origin;
    out.bounds_size = input.bounds_size;
    return out;
}

@fragment
fn fs_underline(input: UnderlineVarying) -> @location(0) vec4<f32> {
    if (any(input.clip_distances < vec4<f32>(0.0))) {
        return vec4<f32>(0.0);
    }

    if ((input.wavy & 0xFFu) == 0u)
    {
        return blend_color(input.color, input.color.a);
    }

    let half_thickness = input.thickness * 0.5;
    let bounds_size_y = input.bounds_size.y;

    let st = (input.position.xy - input.bounds_origin) / bounds_size_y - vec2<f32>(0.0, 0.5);
    let frequency = M_PI_F * UNDERLINE_WAVE_FREQUENCY * input.thickness / bounds_size_y;
    let amplitude = (input.thickness * UNDERLINE_WAVE_HEIGHT_RATIO) / bounds_size_y;

    let sine = sin(st.x * frequency) * amplitude;
    let dSine = cos(st.x * frequency) * amplitude * frequency;
    let distance = (st.y - sine) / sqrt(1.0 + dSine * dSine);
    let distance_in_pixels = distance * bounds_size_y;
    let distance_from_top_border = distance_in_pixels - half_thickness;
    let distance_from_bottom_border = distance_in_pixels + half_thickness;
    let alpha = saturate(0.5 - max(-distance_from_bottom_border, distance_from_top_border));
    return blend_color(input.color, alpha * input.color.a);
}

// --- monochrome sprites --- //

struct MonoSpriteVertexInput {
    @location(0) bounds_origin: vec2<f32>,
    @location(1) bounds_size: vec2<f32>,
    @location(2) content_mask_origin: vec2<f32>,
    @location(3) content_mask_size: vec2<f32>,
    @location(4) color: vec4<f32>,
    @location(5) tile_bounds_origin: vec2<i32>,
    @location(6) tile_bounds_size: vec2<i32>,
    @location(7) transformation_row0: vec2<f32>,
    @location(8) transformation_row1: vec2<f32>,
    @location(9) transformation_translation_pad: vec4<f32>,
}

struct MonoSpriteVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) tile_position: vec2<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
    @location(2) clip_distances: vec4<f32>,
}

@vertex
fn vs_mono_sprite(@builtin(vertex_index) vertex_id: u32, input: MonoSpriteVertexInput) -> MonoSpriteVarying {
    let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));
    let bounds = Bounds(input.bounds_origin, input.bounds_size);
    let content_mask = Bounds(input.content_mask_origin, input.content_mask_size);

    let transform = TransformationMatrix(
        mat2x2<f32>(input.transformation_row0, input.transformation_row1),
        input.transformation_translation_pad.xy
    );

    var out = MonoSpriteVarying();
    out.position = to_device_position_transformed(unit_vertex, bounds, transform);

    out.tile_position = to_tile_position(unit_vertex, input.tile_bounds_origin, input.tile_bounds_size);
    out.color = hsla_to_rgba(Hsla(input.color.x, input.color.y, input.color.z, input.color.w));
    out.clip_distances = distance_from_clip_rect_transformed(unit_vertex, bounds, content_mask, transform);
    return out;
}

@fragment
fn fs_mono_sprite(input: MonoSpriteVarying) -> @location(0) vec4<f32> {
    let sample = textureSample(t_sprite, s_sprite, input.tile_position).r;
    let alpha_corrected = apply_contrast_and_gamma_correction(sample, input.color.rgb, sprite_params.grayscale_enhanced_contrast, sprite_params.gamma_ratios);

    // Alpha clip after using the derivatives.
    if (any(input.clip_distances < vec4<f32>(0.0))) {
        return vec4<f32>(0.0);
    }

    return blend_color(input.color, alpha_corrected);
}

// --- path rasterization (first pass) --- //

struct PathRasterizationVertexInput {
    @location(0) xy_position: vec2<f32>,
    @location(1) st_position: vec2<f32>,
    @location(2) color_tag_colorspace: vec2<u32>,
    @location(3) color_solid: vec4<f32>,
    @location(4) color_grad1: vec4<f32>,
    @location(5) color_grad2: vec4<f32>,
    @location(6) color_grad3: vec4<f32>,
    @location(7) bounds_origin: vec2<f32>,
    @location(8) bounds_size: vec2<f32>,
}

struct PathRasterizationVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) st_position: vec2<f32>,
    @location(1) clip_distances: vec4<f32>,
    @location(2) @interpolate(flat) color_tag_colorspace: vec2<u32>,
    @location(3) @interpolate(flat) color_solid: vec4<f32>,
    @location(4) @interpolate(flat) color_color0: vec4<f32>,
    @location(5) @interpolate(flat) color_color1: vec4<f32>,
    @location(6) @interpolate(flat) bounds_origin: vec2<f32>,
    @location(7) @interpolate(flat) bounds_size: vec2<f32>,
    @location(8) @interpolate(flat) gradient_angle: f32,
    @location(9) @interpolate(flat) stop0_percentage: f32,
    @location(10) @interpolate(flat) stop1_percentage: f32,
}

@vertex
fn vs_path_rasterization(input: PathRasterizationVertexInput) -> PathRasterizationVarying {
    let bounds = Bounds(input.bounds_origin, input.bounds_size);

    // Reconstruct Background from vertex attributes
    // Layout: grad1 = [angle, h0, s0, l0], grad2 = [a0, stop0, h1, s1], grad3 = [l1, a1, stop1, pad]
    let solid = Hsla(input.color_solid.x, input.color_solid.y, input.color_solid.z, input.color_solid.w);
    let gradient_angle = input.color_grad1.x;
    let color0_hsla = Hsla(input.color_grad1.y, input.color_grad1.z, input.color_grad1.w, input.color_grad2.x);
    let stop0_percentage = input.color_grad2.y;
    let color1_hsla = Hsla(input.color_grad2.z, input.color_grad2.w, input.color_grad3.x, input.color_grad3.y);
    let stop1_percentage = input.color_grad3.z;

    var colors: array<LinearColorStop, 2>;
    colors[0] = LinearColorStop(color0_hsla, stop0_percentage);
    colors[1] = LinearColorStop(color1_hsla, stop1_percentage);

    let gradient = prepare_gradient_color(
        input.color_tag_colorspace.x,
        input.color_tag_colorspace.y,
        solid,
        colors
    );

    var out = PathRasterizationVarying();
    out.position = to_device_position_impl(input.xy_position);
    out.st_position = input.st_position;
    out.clip_distances = distance_from_clip_rect_impl(input.xy_position, bounds);
    out.color_tag_colorspace = input.color_tag_colorspace;
    out.color_solid = gradient.solid;
    out.color_color0 = gradient.color0;
    out.color_color1 = gradient.color1;
    out.bounds_origin = input.bounds_origin;
    out.bounds_size = input.bounds_size;
    out.gradient_angle = gradient_angle;
    out.stop0_percentage = stop0_percentage;
    out.stop1_percentage = stop1_percentage;
    return out;
}

@fragment
fn fs_path_rasterization(input: PathRasterizationVarying) -> @location(0) vec4<f32> {
    let dx = dpdx(input.st_position);
    let dy = dpdy(input.st_position);
    if (any(input.clip_distances < vec4<f32>(0.0))) {
        return vec4<f32>(0.0);
    }

    let bounds = Bounds(input.bounds_origin, input.bounds_size);

    var alpha: f32;
    if (length(vec2<f32>(dx.x, dy.x)) < 0.001) {
        alpha = 1.0;
    } else {
        let gradient = 2.0 * input.st_position.xx * vec2<f32>(dx.x, dy.x) - vec2<f32>(dx.y, dy.y);
        let f = input.st_position.x * input.st_position.x - input.st_position.y;
        let distance = f / length(gradient);
        alpha = saturate(0.5 - distance);
    }

    let color = gradient_color_from_prepared(
        input.color_tag_colorspace.x,
        input.color_tag_colorspace.y,
        input.gradient_angle,
        input.stop0_percentage,
        input.stop1_percentage,
        input.position.xy,
        bounds,
        input.color_solid,
        input.color_color0,
        input.color_color1
    );
    return vec4<f32>(color.rgb * color.a * alpha, color.a * alpha);
}

// --- path sprites (second pass) --- //

struct PathSpriteVertexInput {
    @location(0) bounds_origin: vec2<f32>,
    @location(1) bounds_size: vec2<f32>,
}

struct PathVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) texture_coords: vec2<f32>,
}

@vertex
fn vs_path(@builtin(vertex_index) vertex_id: u32, input: PathSpriteVertexInput) -> PathVarying {
    let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));
    let bounds = Bounds(input.bounds_origin, input.bounds_size);
    let device_position = to_device_position(unit_vertex, bounds);
    let screen_position = bounds.origin + unit_vertex * bounds.size;
    let texture_coords = screen_position / globals.viewport_size;

    var out = PathVarying();
    out.position = device_position;
    out.texture_coords = texture_coords;

    return out;
}

@fragment
fn fs_path(input: PathVarying) -> @location(0) vec4<f32> {
    let sample = textureSample(t_sprite, s_sprite, input.texture_coords);
    return sample;
}

// --- polychrome sprites --- //

struct PolySpriteVertexInput {
    @location(0) grayscale_pad: vec2<u32>,
    @location(1) opacity_pad2: vec2<f32>,
    @location(2) bounds_origin: vec2<f32>,
    @location(3) bounds_size: vec2<f32>,
    @location(4) content_mask_origin: vec2<f32>,
    @location(5) content_mask_size: vec2<f32>,
    @location(6) corner_radii: vec4<f32>,
    @location(7) tile_bounds_origin: vec2<i32>,
    @location(8) tile_bounds_size: vec2<i32>,
}

struct PolySpriteVarying {
    @builtin(position) position: vec4<f32>,
    @location(0) tile_position: vec2<f32>,
    @location(1) @interpolate(flat) grayscale: u32,
    @location(2) @interpolate(flat) opacity: f32,
    @location(3) clip_distances: vec4<f32>,
    @location(4) @interpolate(flat) bounds_origin: vec2<f32>,
    @location(5) @interpolate(flat) bounds_size: vec2<f32>,
    @location(6) @interpolate(flat) corner_radii: vec4<f32>,
}

@vertex
fn vs_poly_sprite(@builtin(vertex_index) vertex_id: u32, input: PolySpriteVertexInput) -> PolySpriteVarying {
    let unit_vertex = vec2<f32>(f32(vertex_id & 1u), 0.5 * f32(vertex_id & 2u));
    let bounds = Bounds(input.bounds_origin, input.bounds_size);
    let content_mask = Bounds(input.content_mask_origin, input.content_mask_size);

    var out = PolySpriteVarying();
    out.position = to_device_position(unit_vertex, bounds);
    out.tile_position = to_tile_position(unit_vertex, input.tile_bounds_origin, input.tile_bounds_size);
    out.grayscale = input.grayscale_pad.x;
    out.opacity = input.opacity_pad2.x;
    out.clip_distances = distance_from_clip_rect(unit_vertex, bounds, content_mask);
    out.bounds_origin = input.bounds_origin;
    out.bounds_size = input.bounds_size;
    out.corner_radii = input.corner_radii;
    return out;
}

@fragment
fn fs_poly_sprite(input: PolySpriteVarying) -> @location(0) vec4<f32> {
    let sample = textureSample(t_sprite, s_sprite, input.tile_position);
    // Alpha clip after using the derivatives.
    if (any(input.clip_distances < vec4<f32>(0.0))) {
        return vec4<f32>(0.0);
    }

    let bounds = Bounds(input.bounds_origin, input.bounds_size);
    let corner_radii = Corners(input.corner_radii.x, input.corner_radii.y, input.corner_radii.z, input.corner_radii.w);
    let distance = quad_sdf(input.position.xy, bounds, corner_radii);

    var color = sample;
    if ((input.grayscale & 0xFFu) != 0u) {
        let grayscale = dot(color.rgb, GRAYSCALE_FACTORS);
        color = vec4<f32>(vec3<f32>(grayscale), sample.a);
    }
    return blend_color(color, input.opacity * saturate(0.5 - distance));
}
