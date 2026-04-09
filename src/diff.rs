use serde::Serialize;
use similar::{Algorithm, DiffOp, capture_diff_slices};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TextStats {
    chars: usize,
    spaces: usize,
    chars_with_spaces: usize,
    newlines: usize,
    chars_with_newlines: usize,
    words: usize,
}

impl TextStats {
    fn build(text: &str) -> Self {
        let words = text.split_whitespace().count();

        let without_cr: String = text.chars().filter(|ch| *ch != '\r').collect();
        let chars_with_newlines = without_cr.chars().count();

        let without_newline: String = without_cr.chars().filter(|ch| *ch != '\n').collect();
        let chars_with_spaces = without_newline.chars().count();

        let chars = without_newline
            .chars()
            .filter(|ch| !ch.is_whitespace())
            .count();

        Self {
            chars,
            spaces: chars_with_spaces.saturating_sub(chars),
            chars_with_spaces,
            newlines: chars_with_newlines.saturating_sub(chars_with_spaces),
            chars_with_newlines,
            words,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
enum Token {
    Newline,
    Text(String),
}

#[derive(Clone, PartialEq, Eq)]
struct MarkedToken {
    token: Token,
    highlighted: bool,
}

type DiffSegment = (String, bool);

type DiffRow = (Vec<DiffSegment>, Vec<DiffSegment>);

#[derive(Serialize)]
pub(crate) struct DiffResponse {
    rows: Vec<DiffRow>,
    stats: [TextStats; 2],
}

impl DiffResponse {
    pub(crate) fn build(sequence_a: &str, sequence_b: &str) -> Self {
        let a_tokens = split_text(sequence_a);
        let b_tokens = split_text(sequence_b);
        let mut a_marked = Vec::new();
        let mut b_marked = Vec::new();

        for op in capture_diff_slices(Algorithm::Myers, &a_tokens, &b_tokens) {
            let a_tokens = &a_tokens[op.old_range()];
            let b_tokens = &b_tokens[op.new_range()];
            let deleted_newlines = count_newlines(a_tokens);
            let inserted_newlines = count_newlines(b_tokens);

            append_marked_tokens(
                &mut a_marked,
                a_tokens,
                matches!(op, DiffOp::Delete { .. } | DiffOp::Replace { .. }),
            );
            append_marked_tokens(
                &mut b_marked,
                b_tokens,
                matches!(op, DiffOp::Insert { .. } | DiffOp::Replace { .. }),
            );

            if let Some((output, count)) = if inserted_newlines > deleted_newlines {
                Some((&mut a_marked, inserted_newlines - deleted_newlines))
            } else if deleted_newlines > inserted_newlines {
                Some((&mut b_marked, deleted_newlines - inserted_newlines))
            } else {
                None
            } {
                output.extend((0..count).map(|_| MarkedToken {
                    token: Token::Newline,
                    highlighted: false,
                }));
            }
        }

        Self {
            rows: build_rows(a_marked, b_marked),
            stats: [TextStats::build(sequence_a), TextStats::build(sequence_b)],
        }
    }
}

fn append_marked_tokens(output: &mut Vec<MarkedToken>, tokens: &[Token], highlighted: bool) {
    output.extend(
        tokens
            .iter()
            .cloned()
            .map(|token| MarkedToken { token, highlighted }),
    );
}

fn count_newlines(tokens: &[Token]) -> usize {
    tokens
        .iter()
        .filter(|token| matches!(token, Token::Newline))
        .count()
}

fn build_rows(a_marked: Vec<MarkedToken>, b_marked: Vec<MarkedToken>) -> Vec<DiffRow> {
    let a_rows = split_rows(a_marked);
    let b_rows = split_rows(b_marked);
    let row_count = a_rows.len().max(b_rows.len());
    let mut rows = Vec::with_capacity(row_count);

    for index in 0..row_count {
        rows.push((
            a_rows.get(index).cloned().unwrap_or_default(),
            b_rows.get(index).cloned().unwrap_or_default(),
        ));
    }

    rows
}

fn split_rows(tokens: Vec<MarkedToken>) -> Vec<Vec<DiffSegment>> {
    let mut rows: Vec<Vec<DiffSegment>> = vec![Vec::new()];

    for marked in tokens {
        match marked.token {
            Token::Newline => rows.push(Vec::new()),
            Token::Text(text) => {
                if text.is_empty() {
                    continue;
                }
                let row = rows.last_mut().expect("rows is never empty");

                if let Some(last) = row.last_mut()
                    && last.1 == marked.highlighted
                {
                    last.0.push_str(&text);
                    continue;
                }

                row.push((text, marked.highlighted));
            }
        }
    }

    while rows.last().is_some_and(|row| row.is_empty()) {
        rows.pop();
    }

    rows
}

fn split_text(text: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];
        if rest.starts_with('\n') {
            tokens.push(Token::Newline);
            index += '\n'.len_utf8();
            continue;
        }

        let lower_len = rest
            .bytes()
            .take_while(|byte| byte.is_ascii_lowercase())
            .count();
        if lower_len > 0 {
            tokens.push(Token::Text(rest[..lower_len].to_owned()));
            index += lower_len;
            continue;
        }

        let ch = rest.chars().next().expect("rest is not empty");
        tokens.push(Token::Text(ch.to_string()));
        index += ch.len_utf8();
    }

    tokens
}
