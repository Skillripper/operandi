//! Finding entries in the table: by name, by command line, by filter.
use super::*;

// ------------------------------------------------------------------ lookup

/// The entry named `name` (case-insensitive), by its name, an alias, or an
/// old spelling from `command_names`.
pub fn find(name: &str) -> Option<&'static Entry> {
    let name = name.trim();
    let exact = |n: &str| {
        COMMANDS
            .iter()
            .find(|e| e.names().any(|m| m.eq_ignore_ascii_case(n)))
    };
    exact(name).or_else(|| {
        command_names::renamed_command(name)
            .or_else(|| {
                command_names::COMMAND_EXPANSIONS
                    .iter()
                    .find(|(old, _, _)| old.eq_ignore_ascii_case(name))
                    .map(|(_, new, _)| *new)
            })
            .and_then(exact)
    })
}

/// The entry a whole command line starts with, and the rest of the line
/// (its arguments). Multi-word names win over shorter ones ("Spike Reset"
/// over "Spike").
pub fn find_line(line: &str) -> Option<(&'static Entry, &str)> {
    let line = line.trim();
    // Word boundaries, longest first.
    let mut ends: Vec<usize> = line
        .char_indices()
        .filter(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i)
        .collect();
    ends.push(line.len());
    ends.into_iter()
        .rev()
        .find_map(|end| find(&line[..end]).map(|e| (e, line[end..].trim())))
}

/// The line the app's parser takes for `line` when it names a Modo-named
/// command by that name (UIC-165: `app.undo` runs as `Undo`, `select.convert
/// edge` as `Convert to Edges`). `None` when the line needs no change;
/// `Some(Err)` for arguments the phrase can't take yet.
pub fn parser_spelling(line: &str) -> Option<std::result::Result<String, String>> {
    let (entry, rest) = find_line(line)?;
    let phrase = entry.spelled?;
    let head = &line.trim()[..line.trim().len() - rest.len()].trim();
    if !head.eq_ignore_ascii_case(entry.name) {
        // Typed as the phrase or another alias already.
        return None;
    }
    let value = |key: &str| {
        let r = rest.trim();
        crate::command_names::strip_key(r, key)
            .unwrap_or(r)
            .to_owned()
    };
    Some(match entry.name {
        "cmds.showHelp" => Ok(format!("help {}", value("command")).trim().to_owned()),
        "select.convert" => match value("type").to_ascii_lowercase().as_str() {
            "vertex" | "0" => Ok("Convert to Vertices".into()),
            "edge" | "1" => Ok("Convert to Edges".into()),
            "polygon" | "2" => Ok("Convert to Polygons".into()),
            other => Err(format!(
                "select.convert: unknown type \"{other}\"; use vertex, edge or polygon"
            )),
        },
        "select.loop" => match value("direction").to_ascii_lowercase().as_str() {
            "" => Ok(phrase.into()),
            "next" => Ok("Next Loop".into()),
            "prev" | "previous" => Ok("Previous Loop".into()),
            other => Err(format!(
                "select.loop: unknown direction \"{other}\"; use next or prev"
            )),
        },
        "viewport.fit" => match rest.trim() {
            "" => Ok(phrase.into()),
            _ => match crate::command_args::boolean("selected", &value("selected")) {
                Ok(true) => Ok("Frame Selection".into()),
                Ok(false) => Ok(phrase.into()),
                Err(e) => Err(e),
            },
        },
        _ if rest.trim().is_empty() => Ok(phrase.into()),
        _ => Err(format!(
            "{}: arguments aren't supported yet; use {} with none",
            entry.name, entry.name
        )),
    })
}

/// "Deprecated command name: old is now new" when `name` is an old
/// spelling (plan L5; the alias table is `command_names`).
pub fn deprecation(name: &str) -> Option<String> {
    let name = name.trim();
    let new = command_names::renamed_command(name)
        .or_else(|| command_names::renamed_tool(name))
        .or_else(|| {
            command_names::COMMAND_EXPANSIONS
                .iter()
                .find(|(old, _, _)| old.eq_ignore_ascii_case(name))
                .map(|(_, new, _)| *new)
        })?;
    Some(format!("Deprecated command name: {name} is now {new}"))
}

/// The commands in `filter`'s area, or whose name or alias contains it
/// (all when empty), sorted by area then name.
pub fn matching(filter: &str) -> Vec<&'static Entry> {
    matching_in(COMMANDS.iter(), filter)
}

/// [`matching`] over any set of entries (the run-time registry's too).
pub fn matching_in<'a>(entries: impl Iterator<Item = &'a Entry>, filter: &str) -> Vec<&'a Entry> {
    let filter = filter.trim().to_ascii_lowercase();
    let mut entries: Vec<&Entry> = entries
        .filter(|e| {
            filter.is_empty()
                || e.area.label() == filter
                || e.names().any(|n| n.to_ascii_lowercase().contains(&filter))
        })
        .collect();
    entries.sort_by_key(|e| (e.area, e.kind == Kind::Action, e.name.to_ascii_lowercase()));
    entries
}

/// [`matching`] as one line each: area, usage and help.
pub fn list(filter: &str) -> Vec<String> {
    matching(filter)
        .into_iter()
        .map(|e| format!("[{}] {} — {}", e.area.label(), e.usage(), e.help))
        .collect()
}
