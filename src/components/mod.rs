use tui_realm_stdlib::{Input, List, Phantom};
use tuirealm::{
    command::{
        Cmd,
        CmdResult::{self, Changed},
        Direction, Position,
    },
    event::{Key, KeyEvent},
    props::{
        Alignment, BorderType, Borders, Color, InputType, Style, Table, TableBuilder, TextSpan,
    },
    AttrValue, Attribute, Component, Event, MockComponent,
};

use crate::{
    backend::{Note, Todo},
    AppEvent,
    Msg::{self, NoteSelected},
};

#[derive(MockComponent, Default)]
pub struct PhantomListener {
    component: Phantom,
}

impl Component<Msg, AppEvent> for PhantomListener {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::User(AppEvent::ErrorInitiliazed) => return Some(Msg::AppClose),
            _ => CmdResult::None,
        };
        Some(Msg::None)
    }
}
#[derive(MockComponent)]
pub struct NoteList {
    component: List,
}

impl Default for NoteList {
    fn default() -> Self {
        Self {
            component: List::default()
                .title("Note List", Alignment::Left)
                .highlighted_color(Color::LightYellow)
                .highlighted_str("👉")
                .scroll(true)
                .rewind(true)
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Double)
                        .color(Color::Yellow),
                ),
        }
    }
}

impl Component<Msg, AppEvent> for NoteList {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::NoteListBlur),
            Event::Keyboard(KeyEvent { code: Key::Char('m'), .. }) => {
                return Some(Msg::EditNote(0))
            },
            Event::Keyboard(KeyEvent { code: _, .. }) => self.maybe_scroll_note_list(ev),
            Event::User(AppEvent::NoteLoaded(data)) => {
                if data.is_empty() {
                    return None;
                }
                self.component.attr(
                    Attribute::Content,
                    AttrValue::Table(Self::build_table_list(data)),
                );
                Some(NoteSelected(0))
            }
            _ => None,
        }
    }
}

impl NoteList {
    fn maybe_scroll_note_list(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        if let Changed(state) = maybe_scroll_list(&mut self.component, ev) {
            return Some(NoteSelected(state.unwrap_one().unwrap_usize()));
        }
        None
    }

    fn build_table_list(notes: Vec<Note>) -> Table {
        let mut table = TableBuilder::default();

        notes.iter().enumerate().for_each(|(index, note)| {
            let index_str = format!("{:02}", index);
            
            let row = table
                .add_col(TextSpan::from(index_str).fg(Color::Cyan).italic())
                .add_col(TextSpan::from(" "))
                .add_col(TextSpan::from(note.title().unwrap()));

            if index < notes.len() - 1 {
                row.add_row();
            }
        });

        table.build()
    }
}

#[derive(MockComponent)]
pub struct ShortcutsLegend {
    component: List,
}

impl Default for ShortcutsLegend {
    fn default() -> Self {
        Self {
            component: List::default()
                .title("Key Bindings", Alignment::Left)
                .scroll(false)
                .borders(Borders::default().modifiers(BorderType::Double))
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
                        .add_col(TextSpan::from(" M").bold())
                        .add_col(TextSpan::from("    "))
                        .add_col(TextSpan::from("Modify a note/item"))
                        .add_row()
                        .add_col(TextSpan::from(" S").fg(Color::LightGreen).bold())
                        .add_col(TextSpan::from("    "))
                        .add_col(TextSpan::from("Save note"))
                        .build(),
                ),
        }
    }
}

impl Component<Msg, AppEvent> for ShortcutsLegend {
    fn on(&mut self, _ev: Event<AppEvent>) -> Option<Msg> {
        Some(Msg::None)
    }
}

#[derive(MockComponent)]
pub struct TodoList {
    component: List,
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            component: List::default()
                .title("Item List", Alignment::Left)
                .highlighted_color(Color::LightYellow)
                .highlighted_str("👉")
                .scroll(true)
                .rewind(true)
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Double)
                        .color(Color::Yellow),
                ),
        }
    }
}

impl Component<Msg, AppEvent> for TodoList {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::TodoListBlur),
            Event::Keyboard(KeyEvent { code: _, .. }) => self.maybe_scroll_todo_list(ev),
            _ => None,
        }
    }
}

impl TodoList {
    fn maybe_scroll_todo_list(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        if let Changed(state) = maybe_scroll_list(&mut self.component, ev) {
            return Some(Msg::TodoSelected(state.unwrap_one().unwrap_usize()));
        }
        None
    }

    pub fn build_table_todo(todos: Vec<Todo>) -> Table {
        let mut table = TableBuilder::default();

        todos.iter().enumerate().for_each(|(index, todo)| {
            let (done, space) = match todo.done().unwrap() {
                Some(true) => ("✔️", "  "),
                Some(false) => ("❌", " "),
                None => ("❓", " "),
            };

            let derscription = todo.description().unwrap();
            let row = table
                .add_col(TextSpan::from(done))
                .add_col(TextSpan::from(space))
                .add_col(TextSpan::from(derscription));

            if index < todos.len() - 1 {
                row.add_row();
            }
        });
        table.build()
    }
}

#[derive(MockComponent)]
pub struct EditPopup {
    component: Input,
}

impl Component<Msg, AppEvent> for EditPopup {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => return Some(Msg::TodoListBlur),
            _ => None,
        }
    }
}

impl EditPopup {
    pub fn new(data: &str) -> Self {
        EditPopup {
            component: Input::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::LightYellow),
                )
                .foreground(Color::LightYellow)
                .input_type(InputType::Text)
                //.title("Username", Alignment::Left)
                .value("veeso")
                .invalid_style(Style::default().fg(Color::Red)),
        }
    }
}

fn maybe_scroll_list(list: &mut List, ev: Event<AppEvent>) -> CmdResult {
    match ev {
        Event::Keyboard(KeyEvent {
            code: Key::Down, ..
        }) => list.perform(Cmd::Move(Direction::Down)),
        Event::Keyboard(KeyEvent { code: Key::Up, .. }) => list.perform(Cmd::Move(Direction::Up)),
        Event::Keyboard(KeyEvent {
            code: Key::PageDown,
            ..
        }) => list.perform(Cmd::Scroll(Direction::Down)),
        Event::Keyboard(KeyEvent {
            code: Key::PageUp, ..
        }) => list.perform(Cmd::Scroll(Direction::Up)),
        Event::Keyboard(KeyEvent {
            code: Key::Home, ..
        }) => list.perform(Cmd::GoTo(Position::Begin)),
        Event::Keyboard(KeyEvent { code: Key::End, .. }) => list.perform(Cmd::GoTo(Position::End)),
        _ => CmdResult::None,
    }
}
