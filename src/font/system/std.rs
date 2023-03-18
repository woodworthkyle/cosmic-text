// SPDX-License-Identifier: MIT OR Apache-2.0

use std::{collections::HashMap, sync::Arc};

use fontdb::Family;
use once_cell::sync::Lazy;
use parking_lot::RwLock;

use crate::{Attrs, Font, FontAttrs};

pub static FONT_SYSTEM: Lazy<FontSystem> = Lazy::new(FontSystem::new);

/// Access system fonts
pub struct FontSystem {
    locale: String,
    db: RwLock<fontdb::Database>,
    font_cache: RwLock<HashMap<fontdb::ID, Option<Arc<Font>>>>,
    font_matches_cache: RwLock<HashMap<FontAttrs, Arc<Vec<fontdb::ID>>>>,
}

impl FontSystem {
    /// Create a new [`FontSystem`], that allows access to any installed system fonts
    ///
    /// # Timing
    ///
    /// This function takes some time to run. On the release build, it can take up to a second,
    /// while debug builds can take up to ten times longer. For this reason, it should only be
    /// called once, and the resulting [`FontSystem`] should be shared.
    pub fn new() -> Self {
        Self::new_with_fonts(std::iter::empty())
    }

    pub fn new_with_fonts(fonts: impl Iterator<Item = fontdb::Source>) -> Self {
        let locale = sys_locale::get_locale().unwrap_or_else(|| {
            log::warn!("failed to get system locale, falling back to en-US");
            String::from("en-US")
        });
        log::debug!("Locale: {}", locale);

        let mut db = fontdb::Database::new();
        {
            #[cfg(not(target_arch = "wasm32"))]
            let now = std::time::Instant::now();

            #[cfg(target_os = "redox")]
            db.load_fonts_dir("/ui/fonts");

            db.load_system_fonts();

            for source in fonts {
                db.load_font_source(source);
            }

            //TODO: configurable default fonts
            db.set_monospace_family("Fira Mono");
            db.set_sans_serif_family("Fira Sans");
            db.set_serif_family("DejaVu Serif");

            #[cfg(not(target_arch = "wasm32"))]
            log::info!(
                "Parsed {} font faces in {}ms.",
                db.len(),
                now.elapsed().as_millis()
            );
        }

        Self::new_with_locale_and_db(locale, db)
    }

    /// Create a new [`FontSystem`], manually specifying the current locale and font database.
    pub fn new_with_locale_and_db(locale: String, db: fontdb::Database) -> Self {
        Self {
            locale,
            db: RwLock::new(db),
            font_cache: RwLock::new(HashMap::new()),
            font_matches_cache: RwLock::new(HashMap::new()),
        }
    }

    pub fn locale(&self) -> &str {
        &self.locale
    }

    pub fn get_font(&self, id: fontdb::ID) -> Option<Arc<Font>> {
        if let Some(f) = self.font_cache.read().get(&id) {
            return f.clone();
        }
        let mut font_cache = self.font_cache.write();
        font_cache
            .entry(id)
            .or_insert_with(|| {
                unsafe {
                    self.db.write().make_shared_face_data(id);
                }
                let db = self.db.read();
                let face = db.face(id)?;
                match Font::new(face) {
                    Some(font) => Some(Arc::new(font)),
                    None => {
                        log::warn!("failed to load font '{}'", face.post_script_name);
                        None
                    }
                }
            })
            .clone()
    }

    pub fn get_font_matches(&self, attrs: Attrs) -> Arc<Vec<fontdb::ID>> {
        let font_attrs: FontAttrs = attrs.into();
        if let Some(f) = self.font_matches_cache.read().get(&font_attrs) {
            return f.clone();
        }
        self.font_matches_cache
            .write()
            .entry(font_attrs)
            .or_insert_with(|| {
                #[cfg(not(target_arch = "wasm32"))]
                let now = std::time::Instant::now();

                let ids = self
                    .db
                    .read()
                    .faces()
                    .filter(|face| attrs.matches(face))
                    .map(|face| face.id)
                    .collect::<Vec<_>>();

                #[cfg(not(target_arch = "wasm32"))]
                {
                    let elapsed = now.elapsed();
                    log::debug!("font matches for {:?} in {:?}", attrs, elapsed);
                }

                Arc::new(ids)
            })
            .clone()
    }

    pub fn face_contains_family(&self, id: fontdb::ID, family: &Family) -> bool {
        let db = self.db.read();
        if let Some(face) = db.face(id) {
            let family_name = db.family_name(family);
            face.families.iter().any(|(name, _)| name == family_name)
        } else {
            false
        }
    }

    pub fn face_contains_family_name(&self, id: fontdb::ID, family_name: &str) -> bool {
        if let Some(face) = self.db.read().face(id) {
            face.families.iter().any(|(name, _)| name == family_name)
        } else {
            false
        }
    }

    pub fn face_name(&self, id: fontdb::ID) -> String {
        if let Some(face) = self.db.read().face(id) {
            if let Some((name, _)) = face.families.first() {
                name.clone()
            } else {
                face.post_script_name.clone()
            }
        } else {
            "invalid font id".to_string()
        }
    }
}
