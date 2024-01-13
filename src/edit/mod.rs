#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::{AttrsList, Cursor, TextLayout};
#[cfg(feature = "swash")]
use floem_peniko::Color;

pub use self::editor::*;
mod editor;

#[cfg(feature = "syntect")]
pub use self::syntect::*;
#[cfg(feature = "syntect")]
mod syntect;

#[cfg(feature = "vi")]
pub use self::vi::*;
#[cfg(feature = "vi")]
mod vi;

/// An action to perform on an [`Editor`]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    /// Move cursor to previous character ([Self::Left] in LTR, [Self::Right] in RTL)
    Previous,
    /// Move cursor to next character ([Self::Right] in LTR, [Self::Left] in RTL)
    Next,
    /// Move cursor left
    Left,
    /// Move cursor right
    Right,
    /// Move cursor up
    Up,
    /// Move cursor down
    Down,
    /// Move cursor to start of line
    Home,
    /// Move cursor to end of line
    End,
    /// Move cursor to start of paragraph
    ParagraphStart,
    /// Move cursor to end of paragraph
    ParagraphEnd,
    /// Move cursor up one page
    PageUp,
    /// Move cursor down one page
    PageDown,
    /// Move cursor up or down by a number of pixels
    Vertical(i32),
    /// Escape, clears selection
    Escape,
    /// Insert character at cursor
    Insert(char),
    /// Create new line
    Enter,
    /// Delete text behind cursor
    Backspace,
    /// Delete text in front of cursor
    Delete,
    /// Mouse click at specified position
    Click { x: i32, y: i32 },
    /// Mouse drag to specified position
    Drag { x: i32, y: i32 },
    /// Scroll specified number of lines
    Scroll { lines: i32 },
    /// Move cursor to previous word boundary
    PreviousWord,
    /// Move cursor to next word boundary
    NextWord,
    /// Move cursor to next word boundary to the left
    LeftWord,
    /// Move cursor to next word boundary to the right
    RightWord,
    /// Move cursor to the start of the document
    BufferStart,
    /// Move cursor to the end of the document
    BufferEnd,
}

/// A trait to allow easy replacements of [`Editor`], like `SyntaxEditor`
pub trait Edit {
    /// Get the internal [`Buffer`]
    fn buffer(&self) -> &TextLayout;

    /// Get the internal [`Buffer`], mutably
    fn buffer_mut(&mut self) -> &mut TextLayout;

    /// Get the current cursor position
    fn cursor(&self) -> Cursor;

    /// Get the current selection position
    fn select_opt(&self) -> Option<Cursor>;

    /// Set the current selection position
    fn set_select_opt(&mut self, select_opt: Option<Cursor>);

    /// Shape lines until scroll, after adjusting scroll if the cursor moved
    fn shape_as_needed(&mut self);

    /// Copy selection
    fn copy_selection(&mut self) -> Option<String>;

    /// Delete selection, adjusting cursor and returning true if there was a selection
    // Also used by backspace, delete, insert, and enter when there is a selection
    fn delete_selection(&mut self) -> bool;

    /// Insert a string at the current cursor or replacing the current selection with the given
    /// attributes, or with the previous character's attributes if None is given.
    fn insert_string(&mut self, data: &str, attrs_list: Option<AttrsList>);

    /// Perform an [Action] on the editor
    fn action(&mut self, action: Action);

    /// Draw the editor
    #[cfg(feature = "swash")]
    fn draw<F>(&self, cache: &mut crate::SwashCache, color: Color, f: F)
    where
        F: FnMut(i32, i32, u32, u32, Color);
}
