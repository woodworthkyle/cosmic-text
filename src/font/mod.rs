// SPDX-License-Identifier: MIT OR Apache-2.0
pub(crate) mod fallback;

use alloc::sync::Arc;

pub use self::system::*;
mod system;

/// A font
pub struct Font(FontInner);

#[ouroboros::self_referencing]
#[allow(dead_code)]
struct FontInner {
    id: fontdb::ID,
    data: Arc<dyn AsRef<[u8]> + Send + Sync>,
    #[borrows(data)]
    #[covariant]
    rustybuzz: rustybuzz::Face<'this>,
    // workaround, since ouroboros does not work with #[cfg(feature = "swash")]
    swash: SwashKey,
    metrics: FontMetrics,
}

#[cfg(feature = "swash")]
pub type SwashKey = (u32, swash::CacheKey);

#[cfg(not(feature = "swash"))]
pub type SwashKey = ();

impl Font {
    pub fn new(info: &fontdb::FaceInfo) -> Option<Self> {
        #[allow(unused_variables)]
        let data = match &info.source {
            fontdb::Source::Binary(data) => Arc::clone(data),
            #[cfg(feature = "std")]
            fontdb::Source::File(path) => {
                log::warn!("Unsupported fontdb Source::File('{}')", path.display());
                return None;
            }
            #[cfg(feature = "std")]
            fontdb::Source::SharedFile(_path, data) => Arc::clone(data),
        };

        let face = ttf_parser::Face::parse((*data).as_ref(), info.index).ok()?;
        let metrics = FontMetrics {
            units_per_em: face.units_per_em(),
            is_monospace: face.is_monospaced(),
            ascent: face.ascender() as f32,
            descent: -face.descender() as f32,
            line_gap: face.line_gap() as f32,
            cap_height: face.capital_height().map(|h| h as f32),
            x_height: face.x_height().map(|h| h as f32),
            underline_offset: face.underline_metrics().map(|m| m.position as f32),
            underline_size: face.underline_metrics().map(|m| m.thickness as f32),
            strikeout_offset: face.strikeout_metrics().map(|m| m.position as f32),
            strikeout_size: face.strikeout_metrics().map(|m| m.thickness as f32),
        };

        Some(Self(
            FontInnerTryBuilder {
                id: info.id,
                swash: {
                    #[cfg(feature = "swash")]
                    let swash = {
                        let swash =
                            swash::FontRef::from_index((*data).as_ref(), info.index as usize)?;
                        (swash.offset, swash.key)
                    };
                    #[cfg(not(feature = "swash"))]
                    let swash = ();
                    swash
                },
                data,
                rustybuzz_builder: |data| {
                    rustybuzz::Face::from_slice((**data).as_ref(), info.index).ok_or(())
                },
                metrics,
            }
            .try_build()
            .ok()?,
        ))
    }

    pub fn id(&self) -> fontdb::ID {
        *self.0.borrow_id()
    }

    pub fn data(&self) -> &[u8] {
        (**self.0.borrow_data()).as_ref()
    }

    pub fn rustybuzz(&self) -> &rustybuzz::Face {
        self.0.borrow_rustybuzz()
    }

    #[cfg(feature = "swash")]
    pub fn as_swash(&self) -> swash::FontRef {
        let swash = self.0.borrow_swash();
        swash::FontRef {
            data: self.data(),
            offset: swash.0,
            key: swash.1,
        }
    }

    pub fn metrics(&self) -> &FontMetrics {
        self.0.borrow_metrics()
    }

    // This is used to prevent warnings due to the swash field being unused.
    #[cfg(not(feature = "swash"))]
    #[allow(dead_code)]
    fn as_swash(&self) {
        self.0.borrow_swash();
    }
}

/// Global font metrics.
#[derive(Copy, Clone, Default, Debug)]
pub struct FontMetrics {
    /// Number of font design units per em unit.
    pub units_per_em: u16,
    /// True if the font is monospace.
    pub is_monospace: bool,
    /// Distance from the baseline to the top of the alignment box.
    pub ascent: f32,
    /// Distance from the baseline to the bottom of the alignment box.
    pub descent: f32,
    /// Recommended additional spacing between lines.
    pub line_gap: f32,
    /// Distance from the baseline to the top of a typical English capital.
    pub cap_height: Option<f32>,
    /// Distance from the baseline to the top of the lowercase "x" or
    /// similar character.
    pub x_height: Option<f32>,
    /// Recommended distance from the baseline to the top of an underline
    /// stroke.
    pub underline_offset: Option<f32>,
    /// Recommended thickness of an underline stroke.
    pub underline_size: Option<f32>,
    /// Recommended distance from the baseline to the top of a strikeout
    /// stroke.
    pub strikeout_offset: Option<f32>,
    /// Recommended thickness of a strikeout stroke.
    pub strikeout_size: Option<f32>,
}
