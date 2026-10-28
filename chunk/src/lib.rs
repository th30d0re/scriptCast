//! Text chunker and WAV crossfade helper. The renderer shells out to the
//! `scriptcast-chunk` binary; this library holds the logic so tests can reach it.

pub mod crossfade;
pub mod split;

pub use split::Piece;

/// Render pieces as `{"pieces":[{"text":"...","extended_for_tag":false}]}`.
pub fn pieces_json(pieces: &[Piece]) -> String {
    let mut out = String::from("{\"pieces\":[");
    for (i, piece) in pieces.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str("{\"text\":");
        push_json_string(&mut out, &piece.text);
        out.push_str(",\"extended_for_tag\":");
        out.push_str(if piece.extended_for_tag {
            "true"
        } else {
            "false"
        });
        out.push('}');
    }
    out.push_str("]}");
    out
}

fn push_json_string(out: &mut String, text: &str) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_escapes_quotes_and_controls() {
        let pieces = [Piece {
            text: "a\"b\\c\nd\u{1}".into(),
            extended_for_tag: true,
        }];
        assert_eq!(
            pieces_json(&pieces),
            r#"{"pieces":[{"text":"a\"b\\c\nd\u0001","extended_for_tag":true}]}"#
        );
    }
}
