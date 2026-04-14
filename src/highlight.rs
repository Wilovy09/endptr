/// Simple JSON syntax highlighter.
/// Returns ratatui `Line`s with colored spans — no external parser needed.
///
/// Colors:
///   Keys        → Cyan
///   Strings     → Green
///   Numbers     → Yellow
///   true/false  → Magenta
///   null        → dark gray
///   Brackets/punctuation → White
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

#[derive(Clone, PartialEq)]
enum Ctx {
    Object,
    Array,
}

/// Highlight `json` and return one `Line` per newline.
/// Falls back to plain lines if the input isn't JSON-shaped.
pub fn highlight_json(json: &str) -> Vec<Line<'static>> {
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut spans: Vec<Span<'static>> = Vec::new();
    let mut ctx_stack: Vec<Ctx> = Vec::new();
    // true  → the next string we encounter is an object value, not a key
    let mut after_colon = false;

    let mut chars = json.chars().peekable();

    macro_rules! push_raw {
        ($s:expr) => {
            spans.push(Span::raw($s))
        };
    }
    macro_rules! push_styled {
        ($s:expr, $color:expr) => {
            spans.push(Span::styled($s, Style::new().fg($color)))
        };
    }

    while let Some(ch) = chars.next() {
        match ch {
            '\n' => {
                lines.push(Line::from(std::mem::take(&mut spans)));
            }
            '\r' => {} // skip CR

            // ── String ───────────────────────────────────────────────────────
            '"' => {
                let is_key =
                    ctx_stack.last() == Some(&Ctx::Object) && !after_colon;

                let mut s = String::from('"');
                let mut escaped = false;
                loop {
                    match chars.next() {
                        None => break,
                        Some('\\') if !escaped => {
                            escaped = true;
                            s.push('\\');
                        }
                        Some('"') if !escaped => {
                            s.push('"');
                            break;
                        }
                        Some(c) => {
                            escaped = false;
                            s.push(c);
                        }
                    }
                }

                let color = if is_key { Color::Cyan } else { Color::Green };
                push_styled!(s, color);
                after_colon = false;
            }

            // ── Colon ─────────────────────────────────────────────────────────
            ':' => {
                push_styled!(":".to_string(), Color::White);
                after_colon = true;
            }

            // ── Open bracket ──────────────────────────────────────────────────
            '{' => {
                ctx_stack.push(Ctx::Object);
                after_colon = false;
                push_styled!("{".to_string(), Color::White);
            }
            '[' => {
                ctx_stack.push(Ctx::Array);
                after_colon = false;
                push_styled!("[".to_string(), Color::White);
            }

            // ── Close bracket ─────────────────────────────────────────────────
            '}' => {
                ctx_stack.pop();
                after_colon = false;
                push_styled!("}".to_string(), Color::White);
            }
            ']' => {
                ctx_stack.pop();
                after_colon = false;
                push_styled!("]".to_string(), Color::White);
            }

            // ── Comma ─────────────────────────────────────────────────────────
            ',' => {
                after_colon = false;
                push_styled!(",".to_string(), Color::White);
            }

            // ── Keywords: null / true / false ─────────────────────────────────
            'n' => {
                let rest = read_word(&mut chars, 3);
                let word = format!("n{rest}");
                if word == "null" {
                    push_styled!(word, Color::DarkGray);
                } else {
                    push_raw!(word);
                }
                after_colon = false;
            }
            't' => {
                let rest = read_word(&mut chars, 3);
                let word = format!("t{rest}");
                let color = if word == "true" { Color::Magenta } else { Color::White };
                push_styled!(word, color);
                after_colon = false;
            }
            'f' => {
                let rest = read_word(&mut chars, 4);
                let word = format!("f{rest}");
                let color = if word == "false" { Color::Magenta } else { Color::White };
                push_styled!(word, color);
                after_colon = false;
            }

            // ── Number ────────────────────────────────────────────────────────
            c if c == '-' || c.is_ascii_digit() => {
                let mut s = String::from(c);
                while chars
                    .peek()
                    .map(|p| {
                        p.is_ascii_digit()
                            || *p == '.'
                            || *p == 'e'
                            || *p == 'E'
                            || *p == '+'
                            || *p == '-'
                    })
                    .unwrap_or(false)
                {
                    s.push(chars.next().unwrap());
                }
                push_styled!(s, Color::Yellow);
                after_colon = false;
            }

            // ── Whitespace / other ────────────────────────────────────────────
            c => {
                push_raw!(c.to_string());
            }
        }
    }

    // Flush remaining spans as last line
    if !spans.is_empty() {
        lines.push(Line::from(spans));
    }

    lines
}

/// Try to pretty-print JSON; return highlighted lines.
/// Falls back to plain lines if not valid JSON.
pub fn highlight_json_str(raw: &str) -> Vec<Line<'static>> {
    if raw.trim().is_empty() {
        return vec![];
    }
    let display = if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw) {
        serde_json::to_string_pretty(&v).unwrap_or_else(|_| raw.to_owned())
    } else {
        raw.to_owned()
    };
    highlight_json(&display)
}

/// Copy the clipboard content to system clipboard.
/// Returns error message on failure.
pub fn copy_to_clipboard(text: &str) -> Result<(), String> {
    arboard::Clipboard::new()
        .map_err(|e| e.to_string())?
        .set_text(text)
        .map_err(|e| e.to_string())
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn read_word(chars: &mut std::iter::Peekable<std::str::Chars>, n: usize) -> String {
    let mut s = String::new();
    for _ in 0..n {
        match chars.peek() {
            Some(&c) if c.is_alphabetic() => {
                s.push(c);
                chars.next();
            }
            _ => break,
        }
    }
    s
}
