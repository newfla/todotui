use tui_realm_stdlib::{Container, List};
use tuirealm::{
    event::{Key, KeyEvent},
    props::{Alignment, BorderSides, Borders, Color, Layout, TableBuilder, TextSpan},
    tui::layout::{Constraint, Direction},
    Component, Event, MockComponent, NoUserEvent,
};

use crate::Msg;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(MockComponent)]
pub struct RootContainer {
    component: Container,
}

impl Default for RootContainer {
    fn default() -> Self {
        Self {
            component: Container::default()
                .title(NAME.to_string() + " v." + VERSION, Alignment::Center)
                .layout(
                    Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(&[Constraint::Percentage(50), Constraint::Percentage(50)])
                        .margin(2),
                )
                .children(vec![
                    Box::new(LeftContainer::default()),
                    Box::new(ItemList::default()),
                ]),
        }
    }
}

impl Component<Msg, NoUserEvent> for RootContainer {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::Keyboard(_) => todo!(),
            Event::WindowResize(_, _) => todo!(),
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Paste(_) => todo!(),
            Event::Tick => todo!(),
            Event::None => todo!(),
            Event::User(_) => todo!(),
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
struct LeftContainer {
    component: Container,
}

impl Default for LeftContainer {
    fn default() -> Self {
        Self {
            component: Container::default()
                .borders(Borders::default().sides(BorderSides::NONE))
                .layout(
                    Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(&[Constraint::Percentage(80), Constraint::Percentage(20)])
                        .margin(0),
                )
                .children(vec![
                    Box::new(NoteList::default()),
                    Box::new(ShortcutsLegend::default()),
                ]),
        }
    }
}

impl Component<Msg, NoUserEvent> for LeftContainer {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::Keyboard(_) => todo!(),
            Event::WindowResize(_, _) => todo!(),
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Paste(_) => todo!(),
            Event::Tick => todo!(),
            Event::None => todo!(),
            Event::User(_) => todo!(),
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
struct NoteList {
    component: List,
}

impl Default for NoteList {
    fn default() -> Self {
        Self {
            component: List::default().title("Note List", Alignment::Left),
        }
    }
}

impl Component<Msg, NoUserEvent> for NoteList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::Keyboard(_) => todo!(),
            Event::WindowResize(_, _) => todo!(),
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Paste(_) => todo!(),
            Event::Tick => todo!(),
            Event::None => todo!(),
            Event::User(_) => todo!(),
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
struct ShortcutsLegend {
    component: List,
}

impl Default for ShortcutsLegend {
    fn default() -> Self {
        Self {
            component: List::default()
                .title("Key Bindings", Alignment::Left)
                .scroll(false)
                .rows(
                    TableBuilder::default()
                        .add_col(TextSpan::from(" ESC").fg(Color::LightRed).bold())
                        .add_col(TextSpan::from("  "))
                        .add_col(TextSpan::from("Quit the application"))
                        .add_row()
                        .add_col(TextSpan::from(" TAB").bold())
                        .add_col(TextSpan::from("  "))
                        .add_col(TextSpan::from("Switch focus"))
                        .add_row()
                        .add_col(TextSpan::from(" SPC").bold())
                        .add_col(TextSpan::from("  "))
                        .add_col(TextSpan::from("Cycle between item status"))
                        .add_row()
                        .add_col(TextSpan::from(" A").bold())
                        .add_col(TextSpan::from("    "))
                        .add_col(TextSpan::from("Add a note/item"))
                        .add_row()
                        .add_col(TextSpan::from(" S").fg(Color::LightGreen).bold())
                        .add_col(TextSpan::from("    "))
                        .add_col(TextSpan::from("Save note"))
                        .add_row()
                        .build(),
                ),
        }
    }
}

impl Component<Msg, NoUserEvent> for ShortcutsLegend {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::Keyboard(_) => todo!(),
            Event::WindowResize(_, _) => todo!(),
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Paste(_) => todo!(),
            Event::Tick => todo!(),
            Event::None => todo!(),
            Event::User(_) => todo!(),
        };
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
struct ItemList {
    component: List,
}

impl Default for ItemList {
    fn default() -> Self {
        Self {
            component: List::default().title("Item List", Alignment::Right),
        }
    }
}

impl Component<Msg, NoUserEvent> for ItemList {
    fn on(&mut self, ev: Event<NoUserEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::Keyboard(_) => todo!(),
            Event::WindowResize(_, _) => todo!(),
            Event::FocusGained => todo!(),
            Event::FocusLost => todo!(),
            Event::Paste(_) => todo!(),
            Event::Tick => todo!(),
            Event::None => todo!(),
            Event::User(_) => todo!(),
        };
        Some(Msg::None)
    }
}
