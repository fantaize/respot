mod manager;
mod parse;
mod types;

pub use manager::{CommandManager, CommandResult};
#[allow(unused_imports)] // Re-exported as the error type of parse()
pub use parse::{parse, CommandParseError};
pub use types::*;

use crate::queue::RepeatSetting;
use std::fmt;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub enum Command {
    Quit,
    TogglePlay,
    Stop,
    Previous,
    Next,
    Clear,
    Queue,
    PlayNext,
    Play,
    UpdateLibrary,
    Save,
    SaveCurrent,
    SaveQueue,
    Add,
    AddCurrent,
    Delete,
    Focus(String),
    Seek(SeekDirection),
    VolumeUp(u16),
    VolumeDown(u16),
    Repeat(Option<RepeatSetting>),
    Shuffle(Option<bool>),
    #[cfg(feature = "share_clipboard")]
    Share(TargetMode),
    Back,
    Open(TargetMode),
    Goto(GotoMode),
    Move(MoveMode, MoveAmount),
    Shift(ShiftMode, Option<i32>),
    Search(String),
    Jump(JumpMode),
    Help,
    Lyrics,
    ReloadConfig,
    Noop,
    Insert(InsertSource),
    NewPlaylist(String),
    Sort(SortKey, SortDirection),
    Logout,
    ShowRecommendations(TargetMode),
    Redraw,
    Execute(String),
    Reconnect,
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut repr_tokens = vec![self.basename().to_owned()];
        let mut extras_args = match self {
            Self::Focus(tab) => vec![tab.to_owned()],
            Self::Seek(direction) => vec![direction.to_string()],
            Self::VolumeUp(amount) => vec![amount.to_string()],
            Self::VolumeDown(amount) => vec![amount.to_string()],
            Self::Repeat(mode) => match mode {
                Some(mode) => vec![mode.to_string()],
                None => vec![],
            },
            Self::Shuffle(on) => match on {
                Some(b) => vec![(if *b { "on" } else { "off" }).into()],
                None => vec![],
            },
            #[cfg(feature = "share_clipboard")]
            Self::Share(mode) => vec![mode.to_string()],
            Self::Open(mode) => vec![mode.to_string()],
            Self::Goto(mode) => vec![mode.to_string()],
            Self::Move(mode, amount) => match (mode, amount) {
                (MoveMode::Playing, _) => vec!["playing".to_string()],
                (MoveMode::Up, MoveAmount::Extreme) => vec!["top".to_string()],
                (MoveMode::Down, MoveAmount::Extreme) => vec!["bottom".to_string()],
                (MoveMode::Left, MoveAmount::Extreme) => vec!["leftmost".to_string()],
                (MoveMode::Right, MoveAmount::Extreme) => vec!["rightmost".to_string()],
                (mode, MoveAmount::Float(amount)) => vec![mode.to_string(), amount.to_string()],
                (mode, MoveAmount::Integer(amount)) => vec![mode.to_string(), amount.to_string()],
            },
            Self::Shift(mode, amount) => vec![mode.to_string(), amount.unwrap_or(1).to_string()],
            Self::Search(term) => vec![term.to_owned()],
            Self::Jump(mode) => match mode {
                JumpMode::Previous | JumpMode::Next => vec![],
                JumpMode::Query(term) => vec![term.to_owned()],
            },
            Self::Insert(source) => vec![source.to_string()],
            Self::NewPlaylist(name) => vec![name.to_owned()],
            Self::Sort(key, direction) => vec![key.to_string(), direction.to_string()],
            Self::ShowRecommendations(mode) => vec![mode.to_string()],
            Self::Execute(cmd) => vec![cmd.to_owned()],
            Self::Quit
            | Self::TogglePlay
            | Self::Stop
            | Self::Previous
            | Self::Next
            | Self::Clear
            | Self::Queue
            | Self::PlayNext
            | Self::Play
            | Self::UpdateLibrary
            | Self::Save
            | Self::SaveCurrent
            | Self::SaveQueue
            | Self::Add
            | Self::AddCurrent
            | Self::Delete
            | Self::Back
            | Self::Help
            | Self::Lyrics
            | Self::ReloadConfig
            | Self::Noop
            | Self::Logout
            | Self::Reconnect
            | Self::Redraw => vec![],
        };
        repr_tokens.append(&mut extras_args);
        write!(f, "{}", repr_tokens.join(" "))
    }
}

impl Command {
    pub fn basename(&self) -> &str {
        match self {
            Self::Quit => "quit",
            Self::TogglePlay => "playpause",
            Self::Stop => "stop",
            Self::Previous => "previous",
            Self::Next => "next",
            Self::Clear => "clear",
            Self::Queue => "queue",
            Self::PlayNext => "playnext",
            Self::Play => "play",
            Self::UpdateLibrary => "update",
            Self::Save => "save",
            Self::SaveCurrent => "save current",
            Self::SaveQueue => "save queue",
            Self::Add => "add",
            Self::AddCurrent => "add current",
            Self::Delete => "delete",
            Self::Focus(_) => "focus",
            Self::Seek(_) => "seek",
            Self::VolumeUp(_) => "volup",
            Self::VolumeDown(_) => "voldown",
            Self::Repeat(_) => "repeat",
            Self::Shuffle(_) => "shuffle",
            #[cfg(feature = "share_clipboard")]
            Self::Share(_) => "share",
            Self::Back => "back",
            Self::Open(_) => "open",
            Self::Goto(_) => "goto",
            Self::Move(_, _) => "move",
            Self::Shift(_, _) => "shift",
            Self::Search(_) => "search",
            Self::Jump(JumpMode::Previous) => "jumpprevious",
            Self::Jump(JumpMode::Next) => "jumpnext",
            Self::Jump(JumpMode::Query(_)) => "jump",
            Self::Help => "help",
            Self::Lyrics => "lyrics",
            Self::ReloadConfig => "reload",
            Self::Noop => "noop",
            Self::Insert(_) => "insert",
            Self::NewPlaylist(_) => "newplaylist",
            Self::Sort(_, _) => "sort",
            Self::Logout => "logout",
            Self::ShowRecommendations(_) => "similar",
            Self::Redraw => "redraw",
            Self::Execute(_) => "exec",
            Self::Reconnect => "reconnect",
        }
    }
}
