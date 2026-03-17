use std::sync::Arc;

use cursive::Cursive;
use cursive::view::{View, ViewWrapper};
use cursive::views::{NamedView, ViewRef};
use cursive::XY;

use crate::command::{Command, CommandResult, MoveAmount, MoveMode};
use crate::library::Library;
use crate::model::album::Album;
use crate::model::artist::Artist;
use crate::model::track::Track;
use crate::queue::Queue;
use crate::ui::layout::Layout;

pub trait ListItem: Sync + Send + 'static {
    fn is_playing(&self, queue: &Queue) -> bool;
    fn display_left(&self, library: &Library) -> String;
    fn display_center(&self, _library: &Library) -> String {
        "".to_string()
    }
    fn display_right(&self, library: &Library) -> String;
    fn play(&mut self, queue: &Queue);
    fn play_next(&mut self, queue: &Queue);
    fn queue(&mut self, queue: &Queue);
    fn toggle_saved(&mut self, library: &Library);
    fn save(&mut self, library: &Library);
    fn unsave(&mut self, library: &Library);
    fn open(&self, queue: Arc<Queue>, library: Arc<Library>) -> Option<Box<dyn ViewExt>>;
    fn open_recommendations(
        &mut self,
        _queue: Arc<Queue>,
        _library: Arc<Library>,
    ) -> Option<Box<dyn ViewExt>> {
        None
    }
    fn share_url(&self) -> Option<String>;

    /// Get the album that contains this [ListItem].
    fn album(&self, _queue: &Queue) -> Option<Album> {
        None
    }

    fn artists(&self) -> Option<Vec<Artist>> {
        None
    }

    fn track(&self) -> Option<Track> {
        None
    }

    #[allow(unused_variables)]
    #[inline]
    fn is_saved(&self, library: &Library) -> Option<bool> {
        None
    }

    #[inline]
    fn is_playable(&self) -> bool {
        false
    }

    fn as_listitem(&self) -> Box<dyn ListItem>;
}

pub trait ViewExt: View {
    fn title(&self) -> String {
        "".into()
    }

    fn title_sub(&self) -> String {
        "".into()
    }

    fn on_leave(&self) {}

    fn on_command(&mut self, _s: &mut Cursive, _cmd: &Command) -> Result<CommandResult, String> {
        Ok(CommandResult::Ignored)
    }
}

impl<V: ViewExt> ViewExt for NamedView<V> {
    fn title(&self) -> String {
        self.with_view(|v| v.title()).unwrap_or_default()
    }

    fn title_sub(&self) -> String {
        self.with_view(|v| v.title_sub()).unwrap_or_default()
    }

    fn on_leave(&self) {
        self.with_view(|v| v.on_leave());
    }

    fn on_command(&mut self, s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        self.with_view_mut(move |v| v.on_command(s, cmd)).unwrap()
    }
}

pub trait IntoBoxedViewExt {
    fn into_boxed_view_ext(self) -> Box<dyn ViewExt>;
}

impl<V: ViewExt> IntoBoxedViewExt for V {
    fn into_boxed_view_ext(self) -> Box<dyn ViewExt> {
        Box::new(self)
    }
}

pub struct BoxedViewExt {
    boxed_view: Box<dyn ViewExt>,
}

impl BoxedViewExt {
    pub fn new(view: Box<dyn ViewExt>) -> Self {
        Self { boxed_view: view }
    }
}

impl View for BoxedViewExt {
    fn draw(&self, printer: &cursive::Printer) {
        self.boxed_view.draw(printer);
    }

    fn layout(&mut self, xy: cursive::Vec2) {
        self.boxed_view.layout(xy);
    }

    fn needs_relayout(&self) -> bool {
        self.boxed_view.needs_relayout()
    }

    fn required_size(&mut self, constraint: cursive::Vec2) -> cursive::Vec2 {
        self.boxed_view.required_size(constraint)
    }

    fn on_event(&mut self, event: cursive::event::Event) -> cursive::event::EventResult {
        self.boxed_view.on_event(event)
    }

    fn call_on_any(&mut self, selector: &cursive::view::Selector, callback: cursive::event::AnyCb) {
        self.boxed_view.call_on_any(selector, callback);
    }

    fn focus_view(
        &mut self,
        selector: &cursive::view::Selector,
    ) -> Result<cursive::event::EventResult, cursive::view::ViewNotFound> {
        self.boxed_view.focus_view(selector)
    }

    fn take_focus(
        &mut self,
        source: cursive::direction::Direction,
    ) -> Result<cursive::event::EventResult, cursive::view::CannotFocus> {
        self.boxed_view.take_focus(source)
    }

    fn important_area(&self, view_size: cursive::Vec2) -> cursive::Rect {
        self.boxed_view.important_area(view_size)
    }

    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

impl ViewExt for BoxedViewExt {
    fn title(&self) -> String {
        self.boxed_view.title()
    }

    fn title_sub(&self) -> String {
        self.boxed_view.title_sub()
    }

    fn on_leave(&self) {
        self.boxed_view.on_leave();
    }

    fn on_command(&mut self, s: &mut Cursive, cmd: &Command) -> Result<CommandResult, String> {
        self.boxed_view.on_command(s, cmd)
    }
}

pub trait CursiveExt {
    fn on_layout<F, R>(&mut self, cb: F) -> R
    where
        F: FnOnce(&mut cursive::Cursive, ViewRef<Layout>) -> R;
}

impl CursiveExt for cursive::Cursive {
    fn on_layout<F, R>(&mut self, cb: F) -> R
    where
        F: FnOnce(&mut Self, ViewRef<Layout>) -> R,
    {
        let layout = self
            .find_name::<Layout>("main")
            .expect("Could not find Layout");
        cb(self, layout)
    }
}

pub trait SelectViewExt {
    /// Translates commands (i.e. navigating in lists) to Cursive
    /// `SelectView` actions.
    fn handle_command(&mut self, cmd: &Command) -> Result<CommandResult, String>;
}

impl<T: Send + Sync + 'static> SelectViewExt for cursive::views::SelectView<T> {
    fn handle_command(&mut self, cmd: &Command) -> Result<CommandResult, String> {
        match cmd {
            Command::Move(mode, amount) => {
                let items = self.len();
                match mode {
                    MoveMode::Up => {
                        match amount {
                            MoveAmount::Extreme => self.set_selection(0),
                            MoveAmount::Float(scale) => {
                                let amount = (*self).required_size(XY::default()).y as f32 * scale;
                                self.select_up(amount as usize)
                            }
                            MoveAmount::Integer(amount) => self.select_up(*amount as usize),
                        };
                        Ok(CommandResult::Consumed(None))
                    }
                    MoveMode::Down => {
                        match amount {
                            MoveAmount::Extreme => self.set_selection(items),
                            MoveAmount::Float(scale) => {
                                let amount = (*self).required_size(XY::default()).y as f32 * scale;
                                self.select_down(amount as usize)
                            }
                            MoveAmount::Integer(amount) => self.select_down(*amount as usize),
                        };
                        Ok(CommandResult::Consumed(None))
                    }
                    _ => Ok(CommandResult::Consumed(None)),
                }
            }
            _ => Ok(CommandResult::Ignored),
        }
    }
}
