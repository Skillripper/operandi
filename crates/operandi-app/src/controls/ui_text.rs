//! Text users see: internal spec codes stay out of it (Jason, 2026-11-24).
//!
//! Registry help, form tooltips and greyed-out reasons were written with
//! the behaviour spec's codes in them ("Parent (RG-11/12): ...", "(TK-22,
//! Shift+W)", "UIC-221"). Those mean nothing to a user. Every hover text
//! the toolbox builds goes through [`without_spec_codes`], which drops the
//! codes and keeps the rest ("(Shift+W)"), so new text is covered too.
//!
//! Command lines are internal too (UIC-04: a tooltip gives the name, the
//! shortcut and a usage hint). Tooltips written as "Split · vert.split
//! [key]" or "... (bare mesh.retopo)" lose the command and keep the rest
//! ([`without_command_names`]); [`hover_text`] does both.

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

/// Hover text as users see it: no spec codes and no command lines.
pub(crate) fn hover_text(text: &str) -> String {
    without_command_names(&without_spec_codes(text))
}

/// A word written like a command name: two or more lower-camel words
/// joined by dots (`vert.split`, `operandi.paint.tip`, `tool.set`), with
/// at least one of them three letters or more, so "e.g." and "i.e." stay.
pub(crate) fn is_command_word(word: &str) -> bool {
    let w = word.trim_matches(|c: char| !c.is_ascii_alphanumeric());
    let parts: Vec<&str> = w.split('.').collect();
    parts.len() >= 2
        && parts.iter().any(|p| p.len() >= 3)
        && parts.iter().all(|p| {
            p.chars().next().is_some_and(|c| c.is_ascii_lowercase())
                && p.chars().all(|c| c.is_ascii_alphanumeric())
        })
}

/// A command line's name in words, for a button with no label of its own:
/// its last dotted word split at its capitals (`sculpt.liveWeightEdit` →
/// "Live Weight Edit").
pub(crate) fn command_title(line: &str) -> String {
    let first = line.split_whitespace().next().unwrap_or(line);
    let last = first.rsplit('.').next().unwrap_or(first);
    let mut out = String::new();
    for (i, c) in last.chars().enumerate() {
        if i == 0 {
            out.extend(c.to_uppercase());
        } else {
            if c.is_ascii_uppercase() {
                out.push(' ');
            }
            out.push(c);
        }
    }
    out
}

/// `text` without command lines: a bracketed aside naming one goes
/// ("(bare mesh.retopo)"), and a " · " part that starts with one loses
/// the command and its arguments ("vert.join average:true", "tool.set
/// vert.bevel on.") but keeps what follows ("Click again to drop", a
/// "[key]" binding). Parts left empty go.
pub(crate) fn without_command_names(text: &str) -> String {
    text.split('\n')
        .map(line_without_commands)
        .filter(|l| !l.is_empty() || !text.contains('\n'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn line_without_commands(line: &str) -> String {
    let line = without_command_asides(line);
    let parts: Vec<String> = line
        .split(" · ")
        .map(|part| {
            let words: Vec<&str> = part.split(' ').collect();
            let Some(start) = words.iter().position(|w| !w.is_empty()) else {
                return part.to_owned();
            };
            if !is_command_word(words[start]) {
                return part.to_owned();
            }
            // The command, then its arguments: lower-case words, `key:value`
            // and quoted text, up to a sentence's end or a binding.
            let mut i = start;
            let mut after_tool_set = false;
            while i < words.len() {
                let w = words[i];
                if w.is_empty() {
                    i += 1;
                    continue;
                }
                let argument = i == start
                    || after_tool_set
                    || is_command_word(w)
                    || w.contains(':')
                    || w.starts_with('"')
                    || w.chars().next().is_some_and(|c| c.is_ascii_lowercase());
                if !argument || w.starts_with('[') {
                    break;
                }
                after_tool_set = w == "tool.set";
                i += 1;
                if w.ends_with('.') {
                    break;
                }
            }
            words[i..].join(" ").trim().to_owned()
        })
        .filter(|p| !p.trim().is_empty())
        .collect();
    let joined = parts.join(" · ");
    tidy(&joined).trim().to_owned()
}

/// `line` without bracketed asides that name a command.
fn without_command_asides(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut rest = line;
    while let Some(open) = rest.find('(') {
        let Some(close) = rest[open..].find(')').map(|c| open + c) else {
            break;
        };
        let inside = &rest[open + 1..close];
        if inside.split_whitespace().any(is_command_word) {
            out.push_str(rest[..open].trim_end());
            rest = &rest[close + 1..];
        } else {
            out.push_str(&rest[..=close]);
            rest = &rest[close + 1..];
        }
    }
    out.push_str(rest);
    out
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

    #[test]
    fn command_lines_go_and_the_rest_stays() {
        for (before, after) in [
            ("Split · vert.split  [Ctrl+L] ", "Split · [Ctrl+L]"),
            ("Join Averaged · edge.join average:true", "Join Averaged"),
            (
                "Vertex Bevel · Replace each point · tool.set vert.bevel on. Click again to drop. Shortcut: B",
                "Vertex Bevel · Replace each point · Click again to drop. Shortcut: B",
            ),
            (
                "Weld · drag a point onto another · tool.set DragWeld on",
                "Weld · drag a point onto another",
            ),
            (
                "Automatic Retopology... · Opens Automatic Retopology (bare mesh.retopo)",
                "Automatic Retopology... · Opens Automatic Retopology",
            ),
            (
                "Remesh... · Rebuild: Voxel (sculpt.remesh), or Blocks on an octree (operandi.remesh)",
                "Remesh... · Rebuild: Voxel, or Blocks on an octree",
            ),
            (
                "Inflate · sculpt.inflate · Unassigned\nSwells the surface.",
                "Inflate · Unassigned\nSwells the surface.",
            ),
            ("Text tip · operandi.paint.sculptTip kind:text", "Text tip"),
            (
                "Topology Pen · with Constrain to Background on · operandi.topology.tool",
                "Topology Pen · with Constrain to Background on",
            ),
            (
                "Shift+W · X-Ray · glTF 2.0 · e.g. a cube (i.e. six sides)",
                "Shift+W · X-Ray · glTF 2.0 · e.g. a cube (i.e. six sides)",
            ),
        ] {
            assert_eq!(without_command_names(before), after, "{before}");
        }
        assert_eq!(command_title("sculpt.liveWeightEdit"), "Live Weight Edit");
        assert_eq!(command_title("curves.setColor kind:x"), "Set Color");
    }
}
