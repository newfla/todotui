use tui_realm_stdlib::{Input, List, Phantom};
use tuirealm::{
    command::{
        Cmd,
        CmdResult::{self, Changed},
        Direction, Position,
    },
    event::{Key, KeyEvent, KeyModifiers},
    props::{
        Alignment, BorderType, Borders, Color, InputType, PropPayload, PropValue, Style, Table,
        TableBuilder, TextSpan,
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
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(Msg::NoteListBlur),
            Event::Keyboard(KeyEvent {
                code: Key::Char('e'),
                ..
            }) => Some(Msg::EditNote),
            Event::Keyboard(KeyEvent {
                code: Key::Char('a'),
                ..
            }) => Some(Msg::AddNote),
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                ..
            }) => Some(Msg::RemoveNote),
            Event::Keyboard(KeyEvent { code: _, .. }) => self.maybe_scroll_note_list(ev),
            Event::User(AppEvent::NoteLoaded(data)) => {
                if data.is_empty() {
                    return Some(Msg::None);
                }
                self.component.attr(
                    Attribute::Content,
                    AttrValue::Table(Self::build_table_list(data)),
                );
                Some(NoteSelected(
                    self.component.state().unwrap_one().unwrap_usize(),
                ))
            }
            _ => Some(Msg::None),
        }
    }
}

impl NoteList {
    pub fn new(notes: Vec<Note>, index: usize) -> Self {
        let mut list = NoteList::default();

        list.component.attr(
            Attribute::Content,
            AttrValue::Table(Self::build_table_list(notes)),
        );
        list.component.attr(
            Attribute::Value,
            AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
        );
        list
    }
    fn maybe_scroll_note_list(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        if let Changed(state) = maybe_scroll_list(&mut self.component, ev) {
            return Some(NoteSelected(state.unwrap_one().unwrap_usize()));
        }
        None
    }

    fn build_table_list(notes: Vec<Note>) -> Table {
        let mut table = TableBuilder::default();

        notes.iter().enumerate().for_each(|(index, note)| {
            let index_str = format!("{:03}", index + 1);

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
                        .add_col(TextSpan::from(" ESC").bold())
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
                        .add_col(TextSpan::from("Add note/item"))
                        .add_row()
                        .add_col(TextSpan::from(" E").bold())
                        .add_col(TextSpan::from("    "))
                        .add_col(TextSpan::from("Edit note/item"))
                        .add_row()
                        .add_col(TextSpan::from(" D").bold())
                        .add_col(TextSpan::from("    "))
                        .add_col(TextSpan::from("Delete note/item"))
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
            Event::Keyboard(KeyEvent { code: Key::Tab, .. }) => Some(Msg::TodoListBlur),
            Event::Keyboard(KeyEvent {
                code: Key::Char('e'),
                ..
            }) => Some(Msg::EditTodo),
            Event::Keyboard(KeyEvent {
                code: Key::Char('a'),
                ..
            }) => Some(Msg::AddTodo),
            Event::Keyboard(KeyEvent {
                code: Key::Char('d'),
                ..
            }) => Some(Msg::RemoveTodo),
            Event::Keyboard(KeyEvent { code: _, .. }) => self.maybe_scroll_todo_list(ev),
            _ => Some(Msg::None),
        }
    }
}

impl TodoList {
    pub fn new(todos: Vec<Todo>, index: usize) -> Self {
        let mut list = TodoList::default();

        if !todos.is_empty() {
            list.component.attr(
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(index))),
            );
        }

        list.component.attr(
            Attribute::Content,
            AttrValue::Table(Self::build_table_todo(todos)),
        );
        
        list
    }

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

pub enum EditPopupType {
    Note,
    Todo,
}
#[derive(MockComponent)]
pub struct EditPopup {
    component: Input,
    edit_type: EditPopupType,
}

impl Component<Msg, AppEvent> for EditPopup {
    fn on(&mut self, ev: Event<AppEvent>) -> Option<Msg> {
        //Data edit logic
        let _ = match ev {
            Event::Keyboard(KeyEvent {
                code: Key::Left, ..
            }) => self.perform(Cmd::Move(Direction::Left)),
            Event::Keyboard(KeyEvent {
                code: Key::Right, ..
            }) => self.perform(Cmd::Move(Direction::Right)),
            Event::Keyboard(KeyEvent {
                code: Key::Home, ..
            }) => self.perform(Cmd::GoTo(Position::Begin)),
            Event::Keyboard(KeyEvent { code: Key::End, .. }) => {
                self.perform(Cmd::GoTo(Position::End))
            }
            Event::Keyboard(KeyEvent {
                code: Key::Delete, ..
            }) => self.perform(Cmd::Cancel),
            Event::Keyboard(KeyEvent {
                code: Key::Backspace,
                ..
            }) => self.perform(Cmd::Delete),
            Event::Keyboard(KeyEvent {
                code: Key::Char(ch),
                modifiers: KeyModifiers::NONE,
            }) => self.perform(Cmd::Type(ch)),

            _ => CmdResult::None,
        };
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => match self.edit_type {
                EditPopupType::Note => Some(Msg::CloseEditNote(None)),
                EditPopupType::Todo => Some(Msg::CloseEditTodo(None)),
            },
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let data = self.component.state().unwrap_one().unwrap_string();
                match self.edit_type {
                    EditPopupType::Note => Some(Msg::CloseEditNote(Some(data))),
                    EditPopupType::Todo => Some(Msg::CloseEditTodo(Some(data))),
                }
            }

            _ => Some(Msg::None),
        }
    }
}

impl EditPopup {
    pub fn new(data: &str, title: &str, edit_type: EditPopupType) -> Self {
        EditPopup {
            component: Input::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::LightYellow),
                )
                .foreground(Color::LightYellow)
                .input_type(InputType::Text)
                .title(title, Alignment::Left)
                .value(data)
                .invalid_style(Style::default().fg(Color::Red)),
            edit_type,
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
