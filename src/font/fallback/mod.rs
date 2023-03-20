// SPDX-License-Identifier: MIT OR Apache-2.0

use alloc::sync::Arc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
use fontdb::Family;
use unicode_script::Script;

use crate::{Attrs, Font, FONT_SYSTEM};

use self::platform::*;

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows",)))]
#[path = "other.rs"]
mod platform;

#[cfg(target_os = "macos")]
#[path = "macos.rs"]
mod platform;

#[cfg(target_os = "linux")]
#[path = "unix.rs"]
mod platform;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod platform;

pub struct FontFallbackIter<'a> {
    default_families: &'a [&'a Family<'a>],
    attrs: Attrs<'a>,
    default_i: usize,
    scripts: Vec<Script>,
    script_i: (usize, usize),
    common_i: usize,
    other_i: usize,
    end: bool,
}

impl<'a> FontFallbackIter<'a> {
    pub fn new(
        attrs: Attrs<'a>,
        default_families: &'a [&'a Family<'a>],
        scripts: Vec<Script>,
    ) -> Self {
        Self {
            attrs,
            default_families,
            default_i: 0,
            scripts,
            script_i: (0, 0),
            common_i: 0,
            other_i: 0,
            end: false,
        }
    }

    pub fn check_missing(&mut self, word: &str) {
        if self.end {
            log::debug!(
                "Failed to find any fallback for {:?} locale '{}': '{}'",
                self.scripts,
                FONT_SYSTEM.locale(),
                word
            );
        } else if self.other_i > 0 {
            log::debug!(
                "Failed to find preset fallback for {:?} locale '{}', used  '{}'",
                self.scripts,
                FONT_SYSTEM.locale(),
                word
            );
        } else if !self.scripts.is_empty() && self.common_i > 0 {
            let family = common_fallback()[self.common_i - 1];
            log::debug!(
                "Failed to find script fallback for {:?} locale '{}', used '{}': '{}'",
                self.scripts,
                FONT_SYSTEM.locale(),
                family,
                word
            );
        }
    }
}

impl<'a> Iterator for FontFallbackIter<'a> {
    type Item = Arc<Font>;
    fn next(&mut self) -> Option<Self::Item> {
        while self.default_i < self.default_families.len() {
            self.default_i += 1;
            let default_family = self.default_families[self.default_i - 1];
            if let Some(id) = FONT_SYSTEM.query(*default_family, self.attrs) {
                if let Some(font) = FONT_SYSTEM.get_font(id) {
                    return Some(font);
                }
            }
        }

        while self.script_i.0 < self.scripts.len() {
            let script = self.scripts[self.script_i.0];

            let script_families = script_fallback(script, FONT_SYSTEM.locale());
            while self.script_i.1 < script_families.len() {
                let script_family = script_families[self.script_i.1];
                self.script_i.1 += 1;

                if let Some(id) = FONT_SYSTEM.query(Family::Name(script_family), self.attrs) {
                    if let Some(font) = FONT_SYSTEM.get_font(id) {
                        return Some(font);
                    }
                }
                log::debug!(
                    "failed to find family '{}' for script {:?} and locale '{}'",
                    script_family,
                    script,
                    FONT_SYSTEM.locale(),
                );
            }

            self.script_i.0 += 1;
            self.script_i.1 = 0;
        }

        let common_families = common_fallback();
        while self.common_i < common_families.len() {
            let common_family = common_families[self.common_i];
            self.common_i += 1;

            if let Some(id) = FONT_SYSTEM.query(Family::Name(common_family), self.attrs) {
                if let Some(font) = FONT_SYSTEM.get_font(id) {
                    return Some(font);
                }
            }
            log::debug!("failed to find family '{}'", common_family);
        }

        //TODO: do we need to do this?
        //TODO: do not evaluate fonts more than once!
        let forbidden_families = forbidden_fallback();
        for family in forbidden_families.iter() {
            if let Some(id) = FONT_SYSTEM.query(Family::Name(family), self.attrs) {
                if let Some(font) = FONT_SYSTEM.get_font(id) {
                    return Some(font);
                }
            }
        }

        self.end = true;
        None
    }
}
