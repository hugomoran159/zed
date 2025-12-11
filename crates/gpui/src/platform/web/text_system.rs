use crate::{
    Bounds, DevicePixels, Font, FontFeatures, FontId, FontMetrics, FontRun, FontStyle,
    GlyphId, LineLayout, Pixels, PlatformTextSystem, RenderGlyphParams, SharedString, Size,
    ShapedGlyph, ShapedRun, point, size, SUBPIXEL_VARIANTS_X, SUBPIXEL_VARIANTS_Y,
};
use anyhow::{Context as _, Result};
use collections::HashMap;
use cosmic_text::{
    Attrs, AttrsList, CacheKey, Family, Font as CosmicTextFont, FontFeatures as CosmicFontFeatures,
    fontdb, FontSystem, ShapeBuffer, ShapeLine, SwashCache,
};
use itertools::Itertools;
use parking_lot::RwLock;
use smallvec::SmallVec;
use std::{borrow::Cow, sync::Arc};

#[cfg(feature = "default_fonts")]
const INTER_REGULAR: &[u8] = include_bytes!("fonts/Inter-Regular.ttf");

pub(crate) struct WebTextSystem(RwLock<WebTextSystemState>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct FontKey {
    family: SharedString,
    features: FontFeatures,
}

impl FontKey {
    fn new(family: SharedString, features: FontFeatures) -> Self {
        Self { family, features }
    }
}

struct WebTextSystemState {
    font_system: FontSystem,
    swash_cache: SwashCache,
    scratch: ShapeBuffer,
    loaded_fonts: Vec<LoadedFont>,
    font_ids_by_family_cache: HashMap<FontKey, SmallVec<[FontId; 4]>>,
}

struct LoadedFont {
    font: Arc<CosmicTextFont>,
    features: CosmicFontFeatures,
    is_known_emoji_font: bool,
}

impl WebTextSystem {
    pub(crate) fn new() -> Self {
        let locale = web_sys::window()
            .and_then(|w| w.navigator().language())
            .unwrap_or_else(|| "en-US".to_string());

        let mut db = fontdb::Database::new();

        #[cfg(feature = "default_fonts")]
        {
            web_sys::console::log_1(&format!("Loading embedded Inter font ({} bytes)...", INTER_REGULAR.len()).into());
            db.load_font_source(fontdb::Source::Binary(Arc::new(INTER_REGULAR)));
            let faces: Vec<_> = db.faces().map(|f| f.families.clone()).collect();
            web_sys::console::log_1(&format!("Font database after loading: {:?}", faces).into());
        }

        #[cfg(not(feature = "default_fonts"))]
        {
            web_sys::console::warn_1(&"default_fonts feature is disabled, no fonts embedded".into());
        }

        let font_system = FontSystem::new_with_locale_and_db(locale, db);

        Self(RwLock::new(WebTextSystemState {
            font_system,
            swash_cache: SwashCache::new(),
            scratch: ShapeBuffer::default(),
            loaded_fonts: Vec::new(),
            font_ids_by_family_cache: HashMap::default(),
        }))
    }

    pub async fn load_font_from_url(&self, url: &str) -> Result<()> {
        use wasm_bindgen::JsCast;
        use wasm_bindgen_futures::JsFuture;

        let window = web_sys::window().context("no window available")?;
        let response = JsFuture::from(window.fetch_with_str(url))
            .await
            .map_err(|e| anyhow::anyhow!("fetch failed: {:?}", e))?;

        let response: web_sys::Response = response
            .dyn_into()
            .map_err(|_| anyhow::anyhow!("response is not a Response"))?;

        if !response.ok() {
            anyhow::bail!("failed to fetch font: HTTP {}", response.status());
        }

        let array_buffer = JsFuture::from(
            response
                .array_buffer()
                .map_err(|_| anyhow::anyhow!("failed to get array buffer"))?,
        )
        .await
        .map_err(|e| anyhow::anyhow!("failed to read response: {:?}", e))?;

        let uint8_array = js_sys::Uint8Array::new(&array_buffer);
        let font_bytes = uint8_array.to_vec();

        self.0.write().add_fonts(vec![std::borrow::Cow::Owned(font_bytes)])?;

        Ok(())
    }
}

impl Default for WebTextSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformTextSystem for WebTextSystem {
    fn add_fonts(&self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        self.0.write().add_fonts(fonts)
    }

    fn all_font_names(&self) -> Vec<String> {
        let mut result = self
            .0
            .read()
            .font_system
            .db()
            .faces()
            .filter_map(|face| face.families.first().map(|family| family.0.clone()))
            .collect_vec();
        result.sort();
        result.dedup();
        result
    }

    fn font_id(&self, font: &Font) -> Result<FontId> {
        let mut state = self.0.write();
        let key = FontKey::new(font.family.clone(), font.features.clone());
        let candidates = if let Some(font_ids) = state.font_ids_by_family_cache.get(&key) {
            font_ids.as_slice()
        } else {
            let font_ids = state.load_family(&font.family, &font.features)?;
            state.font_ids_by_family_cache.insert(key.clone(), font_ids);
            state.font_ids_by_family_cache[&key].as_ref()
        };

        if candidates.is_empty() {
            anyhow::bail!("font family '{}' not found", font.family);
        }

        let candidate_properties = candidates
            .iter()
            .map(|font_id| {
                let database_id = state.loaded_font(*font_id).font.id();
                let face_info = state.font_system.db().face(database_id).expect("font face should exist");
                face_info_into_font_properties(face_info)
            })
            .collect::<SmallVec<[_; 4]>>();

        let ix = find_best_match(&candidate_properties, &font_into_font_properties(font))
            .context("requested font family contains no font matching the other parameters")?;

        Ok(candidates[ix])
    }

    fn font_metrics(&self, font_id: FontId) -> FontMetrics {
        let metrics = self
            .0
            .read()
            .loaded_font(font_id)
            .font
            .as_swash()
            .metrics(&[]);

        FontMetrics {
            units_per_em: metrics.units_per_em as u32,
            ascent: metrics.ascent,
            descent: -metrics.descent,
            line_gap: metrics.leading,
            underline_position: metrics.underline_offset,
            underline_thickness: metrics.stroke_size,
            cap_height: metrics.cap_height,
            x_height: metrics.x_height,
            bounding_box: Bounds {
                origin: point(0.0, 0.0),
                size: size(metrics.max_width, metrics.ascent + metrics.descent),
            },
        }
    }

    fn typographic_bounds(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Bounds<f32>> {
        let lock = self.0.read();
        let glyph_metrics = lock.loaded_font(font_id).font.as_swash().glyph_metrics(&[]);
        let glyph_id = glyph_id.0 as u16;
        Ok(Bounds {
            origin: point(0.0, 0.0),
            size: size(
                glyph_metrics.advance_width(glyph_id),
                glyph_metrics.advance_height(glyph_id),
            ),
        })
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        self.0.read().advance(font_id, glyph_id)
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        self.0.read().glyph_for_char(font_id, ch)
    }

    fn glyph_raster_bounds(&self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        self.0.write().raster_bounds(params)
    }

    fn rasterize_glyph(
        &self,
        params: &RenderGlyphParams,
        raster_bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        self.0.write().rasterize_glyph(params, raster_bounds)
    }

    fn layout_line(&self, text: &str, font_size: Pixels, runs: &[FontRun]) -> LineLayout {
        self.0.write().layout_line(text, font_size, runs)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl WebTextSystemState {
    fn loaded_font(&self, font_id: FontId) -> &LoadedFont {
        &self.loaded_fonts[font_id.0]
    }

    fn add_fonts(&mut self, fonts: Vec<Cow<'static, [u8]>>) -> Result<()> {
        let db = self.font_system.db_mut();
        for bytes in fonts {
            match bytes {
                Cow::Borrowed(embedded_font) => {
                    db.load_font_data(embedded_font.to_vec());
                }
                Cow::Owned(bytes) => {
                    db.load_font_data(bytes);
                }
            }
        }
        Ok(())
    }

    fn load_family(
        &mut self,
        name: &str,
        features: &FontFeatures,
    ) -> Result<SmallVec<[FontId; 4]>> {
        let families = self
            .font_system
            .db()
            .faces()
            .filter(|face| face.families.iter().any(|family| name == family.0))
            .map(|face| (face.id, face.post_script_name.clone()))
            .collect::<SmallVec<[_; 4]>>();

        let mut loaded_font_ids = SmallVec::new();
        for (font_id, postscript_name) in families {
            let font = self
                .font_system
                .get_font(font_id)
                .context("Could not load font")?;

            let font_id = FontId(self.loaded_fonts.len());
            loaded_font_ids.push(font_id);
            self.loaded_fonts.push(LoadedFont {
                font,
                features: features.try_into()?,
                is_known_emoji_font: check_is_known_emoji_font(&postscript_name),
            });
        }

        Ok(loaded_font_ids)
    }

    fn advance(&self, font_id: FontId, glyph_id: GlyphId) -> Result<Size<f32>> {
        let glyph_metrics = self.loaded_font(font_id).font.as_swash().glyph_metrics(&[]);
        Ok(Size {
            width: glyph_metrics.advance_width(glyph_id.0 as u16),
            height: glyph_metrics.advance_height(glyph_id.0 as u16),
        })
    }

    fn glyph_for_char(&self, font_id: FontId, ch: char) -> Option<GlyphId> {
        let glyph_id = self.loaded_font(font_id).font.as_swash().charmap().map(ch);
        if glyph_id == 0 {
            None
        } else {
            Some(GlyphId(glyph_id.into()))
        }
    }

    fn raster_bounds(&mut self, params: &RenderGlyphParams) -> Result<Bounds<DevicePixels>> {
        let font = &self.loaded_fonts[params.font_id.0].font;
        let subpixel_shift = point(
            params.subpixel_variant.x as f32 / SUBPIXEL_VARIANTS_X as f32 / params.scale_factor,
            params.subpixel_variant.y as f32 / SUBPIXEL_VARIANTS_Y as f32 / params.scale_factor,
        );
        let image = self
            .swash_cache
            .get_image(
                &mut self.font_system,
                CacheKey::new(
                    font.id(),
                    params.glyph_id.0 as u16,
                    (params.font_size * params.scale_factor).into(),
                    (subpixel_shift.x, subpixel_shift.y.trunc()),
                    cosmic_text::CacheKeyFlags::empty(),
                )
                .0,
            )
            .clone()
            .with_context(|| format!("no image for {params:?} in font {font:?}"))?;
        Ok(Bounds {
            origin: point(image.placement.left.into(), (-image.placement.top).into()),
            size: size(image.placement.width.into(), image.placement.height.into()),
        })
    }

    fn rasterize_glyph(
        &mut self,
        params: &RenderGlyphParams,
        glyph_bounds: Bounds<DevicePixels>,
    ) -> Result<(Size<DevicePixels>, Vec<u8>)> {
        if glyph_bounds.size.width.0 == 0 || glyph_bounds.size.height.0 == 0 {
            anyhow::bail!("glyph bounds are empty");
        }

        let bitmap_size = glyph_bounds.size;
        let font = &self.loaded_fonts[params.font_id.0].font;
        let subpixel_shift = point(
            params.subpixel_variant.x as f32 / SUBPIXEL_VARIANTS_X as f32 / params.scale_factor,
            params.subpixel_variant.y as f32 / SUBPIXEL_VARIANTS_Y as f32 / params.scale_factor,
        );
        let mut image = self
            .swash_cache
            .get_image(
                &mut self.font_system,
                CacheKey::new(
                    font.id(),
                    params.glyph_id.0 as u16,
                    (params.font_size * params.scale_factor).into(),
                    (subpixel_shift.x, subpixel_shift.y.trunc()),
                    cosmic_text::CacheKeyFlags::empty(),
                )
                .0,
            )
            .clone()
            .with_context(|| format!("no image for {params:?} in font {font:?}"))?;

        if params.is_emoji {
            // Convert from RGBA to BGRA.
            for pixel in image.data.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
        }

        Ok((bitmap_size, image.data))
    }

    fn font_id_for_cosmic_id(&mut self, id: cosmic_text::fontdb::ID) -> FontId {
        if let Some(ix) = self
            .loaded_fonts
            .iter()
            .position(|loaded_font| loaded_font.font.id() == id)
        {
            FontId(ix)
        } else {
            let font = self.font_system.get_font(id).unwrap();
            let face = self.font_system.db().face(id).unwrap();

            let font_id = FontId(self.loaded_fonts.len());
            self.loaded_fonts.push(LoadedFont {
                font,
                features: CosmicFontFeatures::new(),
                is_known_emoji_font: check_is_known_emoji_font(&face.post_script_name),
            });

            font_id
        }
    }

    fn layout_line(&mut self, text: &str, font_size: Pixels, font_runs: &[FontRun]) -> LineLayout {
        let mut attrs_list = AttrsList::new(&Attrs::new());
        let mut offs = 0;
        for run in font_runs {
            let loaded_font = self.loaded_font(run.font_id);
            let font = self.font_system.db().face(loaded_font.font.id()).unwrap();

            attrs_list.add_span(
                offs..(offs + run.len),
                &Attrs::new()
                    .metadata(run.font_id.0)
                    .family(Family::Name(&font.families.first().unwrap().0))
                    .stretch(font.stretch)
                    .style(font.style)
                    .weight(font.weight)
                    .font_features(loaded_font.features.clone()),
            );
            offs += run.len;
        }

        let line = ShapeLine::new(
            &mut self.font_system,
            text,
            &attrs_list,
            cosmic_text::Shaping::Advanced,
            4,
        );
        let mut layout_lines = Vec::with_capacity(1);
        line.layout_to_buffer(
            &mut self.scratch,
            font_size.0,
            None,
            cosmic_text::Wrap::None,
            None,
            &mut layout_lines,
            None,
        );
        let layout = layout_lines.first().unwrap();

        let mut runs: Vec<ShapedRun> = Vec::new();
        for glyph in &layout.glyphs {
            let mut font_id = FontId(glyph.metadata);
            let mut loaded_font = self.loaded_font(font_id);
            if loaded_font.font.id() != glyph.font_id {
                font_id = self.font_id_for_cosmic_id(glyph.font_id);
                loaded_font = self.loaded_font(font_id);
            }
            let is_emoji = loaded_font.is_known_emoji_font;

            if glyph.glyph_id == 3 && is_emoji {
                continue;
            }

            let shaped_glyph = ShapedGlyph {
                id: GlyphId(glyph.glyph_id as u32),
                position: point(glyph.x.into(), glyph.y.into()),
                index: glyph.start,
                is_emoji,
            };

            if let Some(last_run) = runs
                .last_mut()
                .filter(|last_run| last_run.font_id == font_id)
            {
                last_run.glyphs.push(shaped_glyph);
            } else {
                runs.push(ShapedRun {
                    font_id,
                    glyphs: vec![shaped_glyph],
                });
            }
        }

        LineLayout {
            font_size,
            width: layout.w.into(),
            ascent: layout.max_ascent.into(),
            descent: layout.max_descent.into(),
            runs,
            len: text.len(),
        }
    }
}

impl TryFrom<&FontFeatures> for CosmicFontFeatures {
    type Error = anyhow::Error;

    fn try_from(features: &FontFeatures) -> Result<Self> {
        let mut result = CosmicFontFeatures::new();
        for feature in features.0.iter() {
            let name_bytes: [u8; 4] = feature
                .0
                .as_bytes()
                .try_into()
                .context("Incorrect feature flag format")?;

            let tag = cosmic_text::FeatureTag::new(&name_bytes);
            result.set(tag, feature.1);
        }
        Ok(result)
    }
}

fn check_is_known_emoji_font(postscript_name: &str) -> bool {
    postscript_name == "NotoColorEmoji"
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FontPropertyStyle {
    Normal,
    Italic,
    Oblique,
}

#[derive(Debug, Clone, Copy)]
struct FontProperties {
    style: FontPropertyStyle,
    weight: f32,
    stretch: f32,
}

impl Default for FontProperties {
    fn default() -> Self {
        Self {
            style: FontPropertyStyle::Normal,
            weight: 400.0,
            stretch: 1.0,
        }
    }
}

fn font_into_font_properties(font: &Font) -> FontProperties {
    FontProperties {
        style: match font.style {
            FontStyle::Normal => FontPropertyStyle::Normal,
            FontStyle::Italic => FontPropertyStyle::Italic,
            FontStyle::Oblique => FontPropertyStyle::Oblique,
        },
        weight: font.weight.0,
        stretch: 1.0,
    }
}

fn face_info_into_font_properties(face_info: &cosmic_text::fontdb::FaceInfo) -> FontProperties {
    FontProperties {
        style: match face_info.style {
            cosmic_text::Style::Normal => FontPropertyStyle::Normal,
            cosmic_text::Style::Italic => FontPropertyStyle::Italic,
            cosmic_text::Style::Oblique => FontPropertyStyle::Oblique,
        },
        weight: face_info.weight.0 as f32,
        stretch: match face_info.stretch {
            cosmic_text::Stretch::UltraCondensed => 0.5,
            cosmic_text::Stretch::ExtraCondensed => 0.625,
            cosmic_text::Stretch::Condensed => 0.75,
            cosmic_text::Stretch::SemiCondensed => 0.875,
            cosmic_text::Stretch::Normal => 1.0,
            cosmic_text::Stretch::SemiExpanded => 1.125,
            cosmic_text::Stretch::Expanded => 1.25,
            cosmic_text::Stretch::ExtraExpanded => 1.5,
            cosmic_text::Stretch::UltraExpanded => 2.0,
        },
    }
}

/// Font matching algorithm following CSS Fonts Level 3 § 5.2
/// https://www.w3.org/TR/css-fonts-3/#font-style-matching
fn find_best_match(candidates: &[FontProperties], query: &FontProperties) -> Option<usize> {
    if candidates.is_empty() {
        return None;
    }

    // Start with all candidates
    let mut dominated: Vec<bool> = vec![false; candidates.len()];

    // Step 4a: Font stretch - find closest match
    let stretch_filter = |props: &FontProperties| -> f32 {
        let diff = props.stretch - query.stretch;
        if query.stretch <= 1.0 {
            // Prefer narrower first, then wider
            if diff <= 0.0 { -diff } else { diff + 1000.0 }
        } else {
            // Prefer wider first, then narrower
            if diff >= 0.0 { diff } else { -diff + 1000.0 }
        }
    };

    let best_stretch_score = candidates
        .iter()
        .enumerate()
        .filter(|(i, _)| !dominated[*i])
        .map(|(_, p)| stretch_filter(p))
        .fold(f32::INFINITY, f32::min);

    for (i, props) in candidates.iter().enumerate() {
        if !dominated[i] && (stretch_filter(props) - best_stretch_score).abs() > 0.001 {
            dominated[i] = true;
        }
    }

    // Step 4b: Font style - preference order based on query style
    let style_preference = |props: &FontProperties| -> u32 {
        match query.style {
            FontPropertyStyle::Italic => match props.style {
                FontPropertyStyle::Italic => 0,
                FontPropertyStyle::Oblique => 1,
                FontPropertyStyle::Normal => 2,
            },
            FontPropertyStyle::Oblique => match props.style {
                FontPropertyStyle::Oblique => 0,
                FontPropertyStyle::Italic => 1,
                FontPropertyStyle::Normal => 2,
            },
            FontPropertyStyle::Normal => match props.style {
                FontPropertyStyle::Normal => 0,
                FontPropertyStyle::Oblique => 1,
                FontPropertyStyle::Italic => 2,
            },
        }
    };

    let best_style_score = candidates
        .iter()
        .enumerate()
        .filter(|(i, _)| !dominated[*i])
        .map(|(_, p)| style_preference(p))
        .min()
        .unwrap_or(0);

    for (i, props) in candidates.iter().enumerate() {
        if !dominated[i] && style_preference(props) != best_style_score {
            dominated[i] = true;
        }
    }

    // Step 4c: Font weight - CSS algorithm
    let weight_distance = |props: &FontProperties| -> f32 {
        let query_weight = query.weight;
        let candidate_weight = props.weight;

        if (candidate_weight - query_weight).abs() < 0.001 {
            return 0.0;
        }

        // For weights 400-500, prefer 500 then search down, then up
        if query_weight >= 400.0 && query_weight <= 500.0 {
            if candidate_weight >= 400.0 && candidate_weight <= 500.0 {
                return (candidate_weight - query_weight).abs();
            }
            if candidate_weight < 400.0 {
                return 400.0 - candidate_weight + 50.0;
            }
            return candidate_weight - 500.0 + 100.0;
        }

        // For weights < 400, prefer lighter then heavier
        if query_weight < 400.0 {
            if candidate_weight <= query_weight {
                return query_weight - candidate_weight;
            }
            return candidate_weight - query_weight + 1000.0;
        }

        // For weights > 500, prefer heavier then lighter
        if candidate_weight >= query_weight {
            return candidate_weight - query_weight;
        }
        query_weight - candidate_weight + 1000.0
    };

    let best_weight_score = candidates
        .iter()
        .enumerate()
        .filter(|(i, _)| !dominated[*i])
        .map(|(_, p)| weight_distance(p))
        .fold(f32::INFINITY, f32::min);

    for (i, props) in candidates.iter().enumerate() {
        if !dominated[i] && (weight_distance(props) - best_weight_score).abs() > 0.001 {
            dominated[i] = true;
        }
    }

    // Return first non-dominated candidate
    dominated.iter().position(|&d| !d)
}
