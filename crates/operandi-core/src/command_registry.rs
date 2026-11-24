//! The command registry: one table of every command Operandi accepts
//! (extensibility plan, phase 0; UI conventions bible UIC-160..164).
//!
//! Every line the command line, a menu, a pie entry, a saved key binding
//! or a script sends goes through `execute` in the app. This table is the
//! authoritative list of the names `execute` understands, with their
//! arguments, units and flags. Dispatch still flows through the app's
//! parsers for now (stage 1); later stages move parsing here and have the
//! MCP bridge read this table instead of keeping its own.
//!
//! Two kinds of entry:
//! - [`Kind::Script`]: dotted script names from the reference name map
//!   (`poly.flip`, `tool.set`). Old spellings resolve through
//!   [`command_names`](crate::command_names) and are reported as
//!   deprecated (plan L5).
//! - [`Kind::Action`]: the plain phrases the Input Map, menus and pies use
//!   (`Grow`, `Top View`). Matched case-insensitively, like the parser.
//!
//! Arguments use the map's units: distances in metres, percents as 0–1
//! (or `50%`), angles in radians (or `30deg`). Panels convert for display.
use crate::command_names;

mod commands;
mod lookup;
#[cfg(test)]
mod tests;
mod tool_ids;

pub use commands::{COMMANDS, EXPORT_FORMATS};
pub use lookup::{deprecation, find, find_line, list, matching, matching_in, parser_spelling};
pub use tool_ids::{TOOL_IDS, TOOL_PHRASES, tool_phrase};

/// The scripting surface's version. Bumped when a command is removed or an
/// argument changes meaning; adding commands or optional arguments doesn't.
pub const API_VERSION: u32 = 1;

/// Script command or Input Map action phrase.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Script,
    Action,
}

/// What a command is about, for `command.list <area>` and reports.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Area {
    Modeling,
    Selection,
    Tools,
    Sculpt,
    View,
    ViewportOptions,
    Items,
    Shading,
    Presets,
    Files,
    Render,
    Preferences,
    Help,
    Debug,
}

impl Area {
    pub const ALL: [Area; 14] = [
        Area::Modeling,
        Area::Selection,
        Area::Tools,
        Area::Sculpt,
        Area::View,
        Area::ViewportOptions,
        Area::Items,
        Area::Shading,
        Area::Presets,
        Area::Files,
        Area::Render,
        Area::Preferences,
        Area::Help,
        Area::Debug,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Area::Modeling => "modeling",
            Area::Selection => "selection",
            Area::Tools => "tools",
            Area::Sculpt => "sculpt",
            Area::View => "view",
            Area::ViewportOptions => "viewport",
            Area::Items => "items",
            Area::Shading => "shading",
            Area::Presets => "presets",
            Area::Files => "files",
            Area::Render => "render",
            Area::Preferences => "preferences",
            Area::Help => "help",
            Area::Debug => "debug",
        }
    }
}

/// An argument's value type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgType {
    Bool,
    Int,
    Float,
    Text,
    /// One of these names (or its index).
    Choice(&'static [&'static str]),
    Path,
    Json,
}

/// The unit a number is given in (the map's script units).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    None,
    /// Metres, or any length unit typed with it (`50cm`).
    Distance,
    /// 0–1, or a value with `%`.
    Percent,
    /// Radians, or degrees typed with a unit (`30deg`).
    Angle,
    /// A plain multiplier (1 = unchanged).
    Factor,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Arg {
    pub name: &'static str,
    pub ty: ArgType,
    pub unit: Unit,
    /// Shown in help; `None` when the command has no single default.
    pub default: Option<&'static str>,
    pub optional: bool,
}

/// A required argument.
pub const fn arg(name: &'static str, ty: ArgType, unit: Unit) -> Arg {
    Arg {
        name,
        ty,
        unit,
        default: None,
        optional: false,
    }
}

/// An optional argument and the value used when it's left out.
pub const fn opt(
    name: &'static str,
    ty: ArgType,
    unit: Unit,
    default: Option<&'static str>,
) -> Arg {
    Arg {
        name,
        ty,
        unit,
        default,
        optional: true,
    }
}

const BOOL: ArgType = ArgType::Bool;
const INT: ArgType = ArgType::Int;
const FLOAT: ArgType = ArgType::Float;
const TEXT: ArgType = ArgType::Text;
const N: Unit = Unit::None;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Flags {
    /// Changes the scene as one undo step when it succeeds.
    pub undoable: bool,
    /// Answers `?` (or reports a value) instead of changing anything.
    pub query: bool,
    /// Only meaningful with the app's window: opens a dialog, panel or
    /// window, or changes how a view looks. Scripts without a window can't
    /// use it.
    pub ui_only: bool,
    /// Starts an interactive tool; its edit is one undo step when dropped.
    pub tool: bool,
    /// Runs with no window through `command_exec` (the MCP bridge's
    /// `command_run` and typed lines alike); its JSON arguments and
    /// handler are the entry's `json`.
    pub headless: bool,
    /// An Operandi name with no match in the reference name map (UIC-165):
    /// kept, but marked.
    pub operandi: bool,
    /// Typed at the keyboard, joins a live Move/Rotate/Scale/Transform in
    /// Items mode instead of being refused by it (PD-Q12).
    pub allowed_during_transform_tool: bool,
    /// Acts outside the scene and can't be undone: save, export, quit,
    /// clearing a log (CMD-01 class "side effect").
    pub side_effect: bool,
    /// Plumbing that shows only in the History tab (CMD-01 class "quiet").
    pub quiet: bool,
}

/// One JSON argument of a headless command: what the MCP bridge's
/// `command_run` and a typed line's words become (`command_exec`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JsonArg {
    pub name: &'static str,
    /// JSON type, e.g. `integer`, `number`, `[x, y, z]`, `string`.
    pub kind: &'static str,
    pub required: bool,
    /// Meaning, unit and default.
    pub about: &'static str,
}

pub const fn jopt(name: &'static str, kind: &'static str, about: &'static str) -> JsonArg {
    JsonArg {
        name,
        kind,
        required: false,
        about,
    }
}
pub const fn jreq(name: &'static str, kind: &'static str, about: &'static str) -> JsonArg {
    JsonArg {
        name,
        kind,
        required: true,
        about,
    }
}

/// The Statistics panel commands' JSON arguments (statistics_edit.rs).
const STATS_ROW: JsonArg = jreq(
    "row",
    "string",
    "A Statistics row, e.g. polygons.vertices:3, edges.border:flat, polygons.part:Wheel.",
);
const STATS_ROW_ARGS: &[JsonArg] = &[STATS_ROW];
const STATS_SELECT_ARGS: &[JsonArg] = &[
    STATS_ROW,
    jopt(
        "action",
        "string",
        "\"add\" (default), \"remove\" or \"only\".",
    ),
];
const STATS_HIDE_ARGS: &[JsonArg] = &[
    STATS_ROW,
    jopt("action", "string", "\"hide\" (default) or \"unhide\"."),
];
const STATS_RENAME_ARGS: &[JsonArg] = &[STATS_ROW, jreq("name", "string", "The new name.")];
const STATS_PASTE_ARGS: &[JsonArg] = &[STATS_ROW, jreq("from", "string", "The copied row.")];

/// Item id argument that defaults to the active item.
const JSON_ITEM: JsonArg = jopt("item", "integer", "Item id; default the active item.");

/// Runs a headless command on the editor alone: the command's current
/// name and its checked JSON arguments. `Ok(Some)` is the one `Command`
/// for the caller to dispatch; `Ok(None)` means it already ran.
pub type EditorHandler = fn(
    &mut crate::Editor,
    &str,
    &serde_json::Map<String, serde_json::Value>,
) -> Result<Option<crate::Command>, String>;

/// What a command that runs without a window takes and what runs it
/// (extensibility plan stage 2, L6).
#[derive(Clone, Copy, Debug)]
pub struct Headless {
    /// The script description (MCP `command_help`, docs/MCP_BRIDGE.md).
    pub about: &'static str,
    pub args: &'static [JsonArg],
    pub run: EditorHandler,
}

#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub name: &'static str,
    /// Other names the parser takes for this command (same behaviour).
    pub aliases: &'static [&'static str],
    pub kind: Kind,
    pub area: Area,
    pub args: &'static [Arg],
    pub flags: Flags,
    /// One line of help.
    pub help: &'static str,
    /// A line that runs it (tests run every example through the parser).
    pub example: &'static str,
    /// The phrase the app's parser takes for this Modo-named command
    /// (`app.undo` runs as `Undo`): see [`parser_spelling`].
    pub spelled: Option<&'static str>,
    /// For a phrase: the Modo invocation of the same button (UIC-165,
    /// specifications/command-names/MODO-BUTTON-INVOCATIONS.txt), e.g.
    /// `tool.set prim.cube on` for `Cube`. Runs through the parser too.
    pub modo: Option<&'static str>,
    /// Set for a command that runs without a window: its JSON arguments
    /// and handler. `flags.headless` says the same.
    pub json: Option<Headless>,
}

const fn entry(kind: Kind, name: &'static str, area: Area, help: &'static str) -> Entry {
    Entry {
        name,
        aliases: &[],
        kind,
        area,
        args: &[],
        flags: Flags {
            undoable: false,
            query: false,
            ui_only: false,
            tool: false,
            headless: false,
            operandi: false,
            allowed_during_transform_tool: false,
            side_effect: false,
            quiet: false,
        },
        spelled: None,
        modo: None,
        json: None,
        help,
        example: name,
    }
}

/// A dotted script command.
pub const fn script(name: &'static str, area: Area, help: &'static str) -> Entry {
    entry(Kind::Script, name, area, help)
}

/// An Input Map action or typed phrase.
pub const fn action(name: &'static str, area: Area, help: &'static str) -> Entry {
    entry(Kind::Action, name, area, help)
}

impl Entry {
    pub const fn aliases(mut self, aliases: &'static [&'static str]) -> Self {
        self.aliases = aliases;
        self
    }
    pub const fn args(mut self, args: &'static [Arg]) -> Self {
        self.args = args;
        self
    }
    pub const fn example(mut self, example: &'static str) -> Self {
        self.example = example;
        self
    }
    pub const fn undo(mut self) -> Self {
        self.flags.undoable = true;
        self
    }
    pub const fn query(mut self) -> Self {
        self.flags.query = true;
        self
    }
    pub const fn ui(mut self) -> Self {
        self.flags.ui_only = true;
        self
    }
    pub const fn tool(mut self) -> Self {
        self.flags.tool = true;
        self
    }
    /// Class "side effect" (CMD-01): outside the scene, not undoable.
    pub const fn side_effect(mut self) -> Self {
        self.flags.side_effect = true;
        self
    }
    /// Class "quiet" (CMD-01): only in the History tab.
    pub const fn quiet(mut self) -> Self {
        self.flags.quiet = true;
        self
    }
    /// The command's class (CMD-01..03, `command_record::class`).
    pub fn class(&self) -> Option<crate::command_record::CommandClass> {
        crate::command_record::class(self)
    }
    pub const fn during_transform_tool(mut self) -> Self {
        self.flags.allowed_during_transform_tool = true;
        self
    }
    pub const fn operandi(mut self) -> Self {
        self.flags.operandi = true;
        self
    }
    /// A Modo-named command the app parser knows by `phrase`, which also
    /// becomes an alias (UIC-165: the phrase is never the only name).
    pub const fn spelled(mut self, phrase: &'static str) -> Self {
        self.spelled = Some(phrase);
        self
    }
    pub const fn modo(mut self, invocation: &'static str) -> Self {
        self.modo = Some(invocation);
        self
    }
    /// Runs on the editor alone (no window) with these JSON arguments and
    /// this handler: a kit's headless command.
    pub const fn on_editor(
        mut self,
        about: &'static str,
        args: &'static [JsonArg],
        run: EditorHandler,
    ) -> Self {
        self.flags.headless = true;
        self.json = Some(Headless { about, args, run });
        self
    }
    /// Runs without a window through `command_exec`, with these JSON
    /// arguments and the built-in handler.
    const fn headless(mut self, about: &'static str, args: &'static [JsonArg]) -> Self {
        self.flags.headless = true;
        self.json = Some(Headless {
            about,
            args,
            run: crate::command_exec::built_in,
        });
        self
    }

    /// Every name this entry answers to.
    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        std::iter::once(self.name)
            .chain(self.spelled)
            .chain(self.aliases.iter().copied())
    }

    /// `name arg:type ...` with optional arguments in brackets.
    pub fn usage(&self) -> String {
        let mut line = self.name.to_owned();
        if self.args.is_empty()
            && let Some(spec) = crate::command_exec::spec(self.name)
        {
            for a in spec.args {
                let one = format!("{}:{}", a.name, a.kind.replace(' ', ""));
                line.push(' ');
                line.push_str(&if a.required { one } else { format!("[{one}]") });
            }
            return line;
        }
        for a in self.args {
            let ty = match a.ty {
                ArgType::Bool => "bool".to_owned(),
                ArgType::Int => "int".to_owned(),
                ArgType::Float => match a.unit {
                    Unit::Distance => "distance".to_owned(),
                    Unit::Percent => "percent".to_owned(),
                    Unit::Angle => "angle".to_owned(),
                    Unit::Factor => "factor".to_owned(),
                    Unit::None => "number".to_owned(),
                },
                ArgType::Text => "text".to_owned(),
                ArgType::Choice(c) => c.join("|"),
                ArgType::Path => "path".to_owned(),
                ArgType::Json => "json".to_owned(),
            };
            let mut one = format!("{}:{ty}", a.name);
            if let Some(d) = a.default {
                one.push_str(&format!("={d}"));
            }
            if a.optional {
                one = format!("[{one}]");
            }
            line.push(' ');
            line.push_str(&one);
        }
        line
    }

    /// The usage line, flags and help, for `help <name>`.
    pub fn describe(&self) -> String {
        let mut flags = Vec::new();
        if self.flags.undoable {
            flags.push("undoable");
        }
        if self.flags.tool {
            flags.push("starts a tool");
        }
        if self.flags.query {
            flags.push("query");
        }
        if self.flags.ui_only {
            flags.push("needs the window");
        }
        if self.flags.headless {
            flags.push("runs without a window");
        }
        if self.flags.operandi {
            flags.push("Operandi-only name");
        }
        if let Some(class) = self.class() {
            flags.push(class.label());
        }
        let mut text = format!("Help: {}", self.usage());
        if !self.aliases.is_empty() {
            text.push_str(&format!(" (also {})", self.aliases.join(", ")));
        }
        text.push_str(&format!(" — {}", self.help));
        if !flags.is_empty() {
            text.push_str(&format!(" [{}]", flags.join(", ")));
        }
        if let Some(modo) = self.modo {
            text.push_str(&format!(" Script form: {modo}."));
        }
        text
    }
}
