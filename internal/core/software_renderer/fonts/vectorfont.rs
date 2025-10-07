// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use alloc::rc::Rc;

use crate::software_renderer::fixed::Fixed;
use crate::software_renderer::PhysicalLength;
use i_slint_common::sharedfontique::fontique;

use super::RenderableVectorGlyph;

type GlyphCacheKey = (u64, u32, PhysicalLength, core::num::NonZeroU16);

struct RenderableGlyphWeightScale;

impl clru::WeightScale<GlyphCacheKey, RenderableVectorGlyph> for RenderableGlyphWeightScale {
    fn weight(&self, _: &GlyphCacheKey, value: &RenderableVectorGlyph) -> usize {
        value.alpha_map.len()
    }
}

type GlyphCache = clru::CLruCache<
    GlyphCacheKey,
    RenderableVectorGlyph,
    std::collections::hash_map::RandomState,
    RenderableGlyphWeightScale,
>;

crate::thread_local!(static GLYPH_CACHE: core::cell::RefCell<GlyphCache>  =
    core::cell::RefCell::new(
        clru::CLruCache::with_config(
            clru::CLruCacheConfig::new(core::num::NonZeroUsize::new(1024 * 1024).unwrap())
                .with_scale(RenderableGlyphWeightScale)
        )
    )
);

pub struct VectorFont {
    font_index: u32,
    font_blob: fontique::Blob<u8>,
    fontdue_font: Rc<fontdue::Font>,
    pixel_size: PhysicalLength,
}

impl VectorFont {
    pub fn new(
        font: fontique::QueryFont,
        fontdue_font: Rc<fontdue::Font>,
        pixel_size: PhysicalLength,
    ) -> Self {
        Self::new_from_blob_and_index(font.blob, font.index, fontdue_font, pixel_size)
    }

    pub fn new_from_blob_and_index(
        font_blob: fontique::Blob<u8>,
        font_index: u32,
        fontdue_font: Rc<fontdue::Font>,
        pixel_size: PhysicalLength,
    ) -> Self {
        Self { font_index, font_blob, fontdue_font, pixel_size }
    }

    pub fn render_glyph(&self, glyph_id: core::num::NonZeroU16) -> Option<RenderableVectorGlyph> {
        GLYPH_CACHE.with(|cache| {
            let mut cache = cache.borrow_mut();

            let cache_key = (self.font_blob.id(), self.font_index, self.pixel_size, glyph_id);

            if let Some(entry) = cache.get(&cache_key) {
                Some(entry.clone())
            } else {
                let (metrics, alpha_map) =
                    self.fontdue_font.rasterize_indexed(glyph_id.get(), self.pixel_size.get() as _);

                let alpha_map: Rc<[u8]> = alpha_map.into();

                let glyph = super::RenderableVectorGlyph {
                    y: Fixed::from_integer(metrics.ymin.try_into().unwrap()),
                    width: PhysicalLength::new(metrics.width.try_into().unwrap()),
                    height: PhysicalLength::new(metrics.height.try_into().unwrap()),
                    alpha_map,
                    pixel_stride: metrics.width.try_into().unwrap(),
                };

                cache.put_with_weight(cache_key, glyph.clone()).ok();
                Some(glyph)
            }
        })
    }
}
