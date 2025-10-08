// Copyright © SixtyFPS GmbH <info@slint.dev>
// SPDX-License-Identifier: GPL-3.0-only OR LicenseRef-Slint-Royalty-free-2.0 OR LicenseRef-Slint-Software-3.0

use core::cell::RefCell;

use alloc::boxed::Box;
use alloc::rc::Rc;
use std::collections::HashMap;

use i_slint_common::sharedfontique::{self, fontique};

crate::thread_local! {
    static FONTDUE_FONTS: RefCell<HashMap<(u64, u32), Rc<fontdue::Font>>> = Default::default();
}

pub fn get_or_create_fontdue_font(blob: &fontique::Blob<u8>, index: u32) -> Rc<fontdue::Font> {
    FONTDUE_FONTS.with(|font_cache| {
        font_cache
            .borrow_mut()
            .entry((blob.id(), index))
            .or_insert_with(move || {
                fontdue::Font::from_bytes(
                    blob.data(),
                    fontdue::FontSettings {
                        collection_index: index,
                        scale: 40.,
                        ..Default::default()
                    },
                )
                .expect("fatal: fontdue is unable to parse truetype font")
                .into()
            })
            .clone()
    })
}

pub fn register_font_from_memory(data: &'static [u8]) -> Result<(), Box<dyn std::error::Error>> {
    sharedfontique::get_collection().register_fonts(data.to_vec().into(), None);
    Ok(())
}

#[cfg(not(target_family = "wasm"))]
pub fn register_font_from_path(path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
    let requested_path = path.canonicalize().unwrap_or_else(|_| path.into());
    let contents = std::fs::read(requested_path)?;
    sharedfontique::get_collection().register_fonts(contents.into(), None);
    Ok(())
}
