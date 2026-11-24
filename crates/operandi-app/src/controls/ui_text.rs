//! Text users see: internal spec codes stay out of it (Jason, 2026-11-24).
//!
//! Registry help, form tooltips and greyed-out reasons were written with
//! the behaviour spec's codes in them ("Parent (RG-11/12): ...", "(TK-22,
//! Shift+W)", "UIC-221"). Those mean nothing to a user. Every hover text
//! the toolbox builds goes through [`without_spec_codes`], which drops the
//! codes and keeps the rest ("(Shift+W)"), so new text is covered too.

/// `text` with spec codes removed: two to four capital letters, a hyphen,
/// an optional `Q`, digits and an optional letter, with ranges or lists
/// joined by `/`, `..`, `-` or `, ` (`RG-11/12`, `SK-20..23`,
/// `RG-11/RG-30`, `LAY-Q47`). Words such as "X-Ray" or "UV-mapped" stay:
/// no digit follows their hyphen. Brackets left empty go, and the
/// separators and spaces left behind are tidied.
pub(crate) fn without_spec_codes(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        if let Some(end) = code_at(&chars, i) {
            i = end;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    tidy(&out)
}

/// The end of a spec code starting at `i`, if one does.
fn code_at(chars: &[char], i: usize) -> Option<usize> {
    // Not inside a longer word ("XYZ-1" in "AXYZ-1" is not a code).
    if i > 0 && (chars[i - 1].is_alphanumeric() || chars[i - 1] == '_') {
        return None;
    }
    let mut j = i;
    while j < chars.len() && chars[j].is_ascii_uppercase() && j - i < 5 {
        j += 1;
    }
    let letters = j - i;
    if !(2..=4).contains(&letters) || chars.get(j) != Some(&'-') {
        return None;
    }
    let mut end = number_at(chars, j + 1)?;
    // Ranges and lists of further numbers or codes: 11/12, 20..23,
    // 11/RG-30, 60-62, "TK-20, TK-21".
    loop {
        let after = match chars.get(end) {
            Some('/') | Some('-') => end + 1,
            Some('.') if chars.get(end + 1) == Some(&'.') => end + 2,
            Some(',') if chars.get(end + 1) == Some(&' ') => end + 2,
            _ => break,
        };
        if let Some(next) = number_at(chars, after) {
            end = next;
        } else if let Some(next) = code_at(chars, after) {
            end = next;
        } else {
            break;
        }
    }
    Some(end)
}

/// The end of `Q`?digits letter? at `i`.
fn number_at(chars: &[char], i: usize) -> Option<usize> {
    let mut j = i;
    if chars.get(j) == Some(&'Q') {
        j += 1;
    }
    let digits = j;
    while j < chars.len() && chars[j].is_ascii_digit() {
        j += 1;
    }
    if j == digits {
        return None;
    }
    if j < chars.len()
        && chars[j].is_ascii_alphabetic()
        && !chars.get(j + 1).is_some_and(|c| c.is_alphanumeric())
    {
        j += 1;
    }
    Some(j)
}

/// Empty brackets, doubled or dangling separators and spaces left by the
/// removed codes.
fn tidy(text: &str) -> String {
    let mut s = text.to_owned();
    loop {
        let before = s.clone();
        for (from, to) in [
            ("( ", "("),
            (" )", ")"),
            ("(, ", "("),
            ("(; ", "("),
            ("(/ ", "("),
            ("(/", "("),
            ("(,", "("),
            ("(;", "("),
            (", )", ")"),
            ("; )", ")"),
            (",)", ")"),
            (";)", ")"),
            ("/)", ")"),
            (" ()", ""),
            ("()", ""),
            (" ,", ","),
            (" ;", ";"),
            (" .", "."),
            (" :", ":"),
            (",,", ","),
            ("  ", " "),
            ("· ·", "·"),
        ] {
            s = s.replace(from, to);
        }
        if s == before {
            break;
        }
    }
    // A sentence that began with a code ("UIC-221: ...", ": ...").
    let trimmed = s.trim_matches(|c: char| c == ' ' || c == ':' || c == ',' || c == ';');
    trimmed.to_owned()
}

/// Whether `text` still holds a spec code (tests).
#[cfg(test)]
pub(crate) fn has_spec_code(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    (0..chars.len()).any(|i| code_at(&chars, i).is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_go_and_the_rest_stays() {
        for (before, after) in [
            (
                "Parent (RG-11/12): the last picked item becomes the parent",
                "Parent: the last picked item becomes the parent",
            ),
            (
                "Keys the selected items' position (TK-22, Shift+W).",
                "Keys the selected items' position (Shift+W).",
            ),
            (
                "Steps back one frame (TK-63; comma in the Animate layout).",
                "Steps back one frame (comma in the Animate layout).",
            ),
            (
                "Bind (SK-20..23): weights for each joint",
                "Bind: weights for each joint",
            ),
            (
                "Compensation (RG-11/RG-30): rig edits",
                "Compensation: rig edits",
            ),
            (
                "No form has this id; it shows nothing (UIC-221)",
                "No form has this id; it shows nothing",
            ),
            (
                "Opens the Paint tab (PNT-85); it then stays",
                "Opens the Paint tab; it then stays",
            ),
            (
                "Adds a modifier (RG-60..62): Expression",
                "Adds a modifier: Expression",
            ),
            (
                "Variant list (MB-Q7) · Shift: Bevel",
                "Variant list · Shift: Bevel",
            ),
            (
                "Shift+W · X-Ray · glTF 2.0 · UV-mapped",
                "Shift+W · X-Ray · glTF 2.0 · UV-mapped",
            ),
        ] {
            assert_eq!(without_spec_codes(before), after, "{before}");
        }
    }
}
