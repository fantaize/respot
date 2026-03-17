use std::sync::Arc;

use cursive::Cursive;
use cursive::event::{Event, EventResult, MouseButton, MouseEvent};
use cursive::theme::{BaseColor, Color, ColorStyle, ColorType, Effect, PaletteColor};
use cursive::traits::View;
use cursive::vec::Vec2;
use cursive::Printer;

use crate::command::{Command, CommandResult, MoveAmount, MoveMode};
use crate::lyrics::{self, Lyrics};
use crate::model::playable::Playable;
use crate::queue::Queue;
use crate::traits::ViewExt;

pub struct LyricsView {
    queue: Arc<Queue>,
    lyrics: Option<Lyrics>,
    current_track_uri: Option<String>,
    scroll_offset: usize,
    last_size: Vec2,
    manual_scroll: bool,
}

impl LyricsView {
    pub fn new(queue: Arc<Queue>) -> Self {
        let mut lv = Self {
            queue,
            lyrics: None,
            current_track_uri: None,
            scroll_offset: 0,
            last_size: Vec2::zero(),
            manual_scroll: false,
        };
        lv.refresh();
        lv
    }

    fn refresh(&mut self) {
        let current = self.queue.get_current();
        let uri = current.as_ref().map(|p| p.uri());

        if uri == self.current_track_uri {
            return;
        }

        self.current_track_uri = uri;
        self.scroll_offset = 0;
        self.manual_scroll = false;

        self.lyrics = current
            .as_ref()
            .filter(|p| matches!(p, Playable::Track(_)))
            .and_then(lyrics::fetch);
    }

    /// Find the index of the line currently being sung based on playback progress.
    fn current_line_index(&self) -> Option<usize> {
        let lyrics = self.lyrics.as_ref()?;
        if !lyrics.synced {
            return None;
        }

        let progress = self.queue.get_spotify().get_current_progress();
        let mut current = 0;
        for (i, line) in lyrics.lines.iter().enumerate() {
            if line.start <= progress {
                current = i;
            }
        }
        Some(current)
    }

    /// Get the effective scroll offset (auto or manual).
    fn effective_scroll(&self, height: usize) -> usize {
        if !self.manual_scroll {
            if let Some(idx) = self.current_line_index() {
                idx.saturating_sub(height / 2)
            } else {
                self.scroll_offset
            }
        } else {
            self.scroll_offset
        }
    }

    /// Seek playback to the timestamp of the line at the given screen y position.
    fn seek_to_line(&self, screen_y: usize) {
        let lyrics = match &self.lyrics {
            Some(l) if l.synced => l,
            _ => return,
        };

        let scroll = self.effective_scroll(self.last_size.y);
        let line_idx = scroll + screen_y;
        if let Some(line) = lyrics.lines.get(line_idx) {
            self.queue.get_spotify().seek(line.start.as_millis() as u32);
        }
    }
}

impl View for LyricsView {
    fn draw(&self, printer: &Printer<'_, '_>) {
        let height = printer.size.y;

        match &self.lyrics {
            None => {
                let msg = if self.current_track_uri.is_some() {
                    "No lyrics available."
                } else {
                    "No track playing."
                };
                printer.with_effect(Effect::Italic, |p| p.print((0, 0), msg));
            }
            Some(lyrics) => {
                let current_idx = self.current_line_index();
                let scroll = self.effective_scroll(height);

                // Current line: bold blue
                let playing_style = ColorStyle::new(
                    ColorType::Color(
                        *printer
                            .theme
                            .palette
                            .custom("playing")
                            .unwrap_or(&Color::Light(BaseColor::Blue)),
                    ),
                    ColorType::Palette(PaletteColor::Background),
                );

                // Played lines: bold white
                let played_style = ColorStyle::new(
                    ColorType::Color(Color::Light(BaseColor::White)),
                    ColorType::Palette(PaletteColor::Background),
                );

                for y in 0..height {
                    let line_idx = scroll + y;
                    if line_idx >= lyrics.lines.len() {
                        break;
                    }
                    let line = &lyrics.lines[line_idx];
                    let is_current = current_idx == Some(line_idx);
                    let is_played = current_idx.is_some_and(|ci| line_idx < ci);

                    if is_current {
                        printer.with_color(playing_style, |p| {
                            p.print_hline((0, y), printer.size.x, " ");
                            p.with_effect(Effect::Bold, |p| {
                                p.print((0, y), &line.text);
                            });
                        });
                    } else if is_played {
                        printer.with_color(played_style, |p| {
                            p.with_effect(Effect::Bold, |p| {
                                p.print((0, y), &line.text);
                            });
                        });
                    } else {
                        printer.print((0, y), &line.text);
                    }
                }
            }
        }
    }

    fn layout(&mut self, size: Vec2) {
        self.last_size = size;
    }

    fn required_size(&mut self, constraint: Vec2) -> Vec2 {
        constraint
    }

    fn needs_relayout(&self) -> bool {
        true
    }

    fn on_event(&mut self, event: Event) -> EventResult {
        match event {
            // Click to seek to a lyric line
            Event::Mouse {
                event: MouseEvent::Press(MouseButton::Left),
                position,
                offset,
            } => {
                if let Some(y) = position.checked_sub(offset).map(|p| p.y) {
                    self.seek_to_line(y);
                    self.manual_scroll = false;
                    return EventResult::consumed();
                }
                EventResult::Ignored
            }
            // Scroll wheel
            Event::Mouse {
                event: MouseEvent::WheelUp,
                ..
            } => {
                self.manual_scroll = true;
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
                EventResult::consumed()
            }
            Event::Mouse {
                event: MouseEvent::WheelDown,
                ..
            } => {
                self.manual_scroll = true;
                let max = self
                    .lyrics
                    .as_ref()
                    .map(|l| l.lines.len().saturating_sub(1))
                    .unwrap_or(0);
                self.scroll_offset = (self.scroll_offset + 1).min(max);
                EventResult::consumed()
            }
            _ => EventResult::Ignored,
        }
    }
}

impl ViewExt for LyricsView {
    fn title(&self) -> String {
        "Lyrics".to_string()
    }

    fn on_command(&mut self, _s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        self.refresh();

        match cmd {
            Command::Move(mode, amount) => {
                let max_scroll = self
                    .lyrics
                    .as_ref()
                    .map(|l| l.lines.len().saturating_sub(1))
                    .unwrap_or(0);

                match mode {
                    MoveMode::Up => {
                        self.manual_scroll = true;
                        let delta = match amount {
                            MoveAmount::Extreme => self.scroll_offset,
                            MoveAmount::Float(s) => (self.last_size.y as f32 * s) as usize,
                            MoveAmount::Integer(n) => *n as usize,
                        };
                        self.scroll_offset = self.scroll_offset.saturating_sub(delta);
                    }
                    MoveMode::Down => {
                        self.manual_scroll = true;
                        let delta = match amount {
                            MoveAmount::Extreme => max_scroll,
                            MoveAmount::Float(s) => (self.last_size.y as f32 * s) as usize,
                            MoveAmount::Integer(n) => *n as usize,
                        };
                        self.scroll_offset = (self.scroll_offset + delta).min(max_scroll);
                    }
                    MoveMode::Playing => {
                        self.manual_scroll = false;
                    }
                    _ => {}
                }
                Ok(CommandResult::Consumed(None))
            }
            _ => Ok(CommandResult::Ignored),
        }
    }
}
