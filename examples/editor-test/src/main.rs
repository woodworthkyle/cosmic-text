// SPDX-License-Identifier: MIT OR Apache-2.0

use cosmic_text::{Action, Color, Edit, Editor, SwashCache, TextLayout, FONT_SYSTEM};
use orbclient::{EventOption, Renderer, Window, WindowFlag};
use std::{env, fs, process, time::Instant};
use unicode_segmentation::UnicodeSegmentation;

fn redraw(window: &mut Window, editor: &mut Editor, swash_cache: &mut SwashCache) {
    let bg_color = orbclient::Color::rgb(0x34, 0x34, 0x34);
    let font_color = Color::rgb(0xFF, 0xFF, 0xFF);

    editor.shape_as_needed();
    if editor.buffer().redraw() {
        let instant = Instant::now();

        window.set(bg_color);

        editor.draw(swash_cache, font_color, |x, y, w, h, color| {
            window.rect(x, y, w, h, orbclient::Color { data: color.0 });
        });

        window.sync();

        editor.buffer_mut().set_redraw(false);

        let duration = instant.elapsed();
        log::debug!("redraw: {:?}", duration);
    }
}

fn main() {
    env_logger::init();

    let mut window = Window::new_flags(
        -1,
        -1,
        1024,
        768,
        &format!("COSMIC TEXT - {}", FONT_SYSTEM.locale()),
        &[WindowFlag::Async],
    )
    .unwrap();

    let mut buffer = TextLayout::new();
    buffer.set_size(window.width() as f32, window.height() as f32);

    let mut editor = Editor::new(buffer);

    let mut swash_cache = SwashCache::new();

    let text = if let Some(arg) = env::args().nth(1) {
        fs::read_to_string(&arg).expect("failed to open file")
    } else {
        #[cfg(feature = "mono")]
        let default_text = include_str!("../../../sample/mono.txt");
        #[cfg(not(feature = "mono"))]
        let default_text = include_str!("../../../sample/proportional.txt");
        default_text.to_string()
    };

    let test_start = Instant::now();

    //TODO: support bidi
    for line in text.lines() {
        log::debug!("Line {:?}", line);

        for grapheme in line.graphemes(true) {
            for c in grapheme.chars() {
                log::trace!("Insert {:?}", c);

                // Test backspace of character
                {
                    let cursor = editor.cursor();
                    editor.action(Action::Insert(c));
                    editor.action(Action::Backspace);
                    assert_eq!(cursor, editor.cursor());
                }

                // Finally, normal insert of character
                editor.action(Action::Insert(c));
            }

            // Test delete of EGC
            {
                let cursor = editor.cursor();
                editor.action(Action::Previous);
                editor.action(Action::Delete);
                for c in grapheme.chars() {
                    editor.action(Action::Insert(c));
                }
                assert_eq!(
                    (cursor.line, cursor.index),
                    (editor.cursor().line, editor.cursor().index)
                );
            }
        }

        // Test backspace of newline
        {
            let cursor = editor.cursor();
            editor.action(Action::Enter);
            editor.action(Action::Backspace);
            assert_eq!(cursor, editor.cursor());
        }

        // Test delete of newline
        {
            let cursor = editor.cursor();
            editor.action(Action::Enter);
            editor.action(Action::Previous);
            editor.action(Action::Delete);
            assert_eq!(cursor, editor.cursor());
        }

        // Finally, normal enter
        editor.action(Action::Enter);

        redraw(&mut window, &mut editor, &mut swash_cache);

        for event in window.events() {
            if let EventOption::Quit(_) = event.to_option() {
                process::exit(1)
            }
        }
    }

    let test_elapsed = test_start.elapsed();
    log::info!("Test completed in {:?}", test_elapsed);

    let mut wrong = 0;
    for (line_i, line) in text.lines().enumerate() {
        let buffer_line = &editor.buffer().lines[line_i];
        if buffer_line.text() != line {
            log::error!("line {}: {:?} != {:?}", line_i, buffer_line.text(), line);
            wrong += 1;
        }
    }
    if wrong == 0 {
        log::info!("All lines matched!");
        process::exit(0);
    } else {
        log::error!("{} lines did not match!", wrong);
        process::exit(1);
    }
}
