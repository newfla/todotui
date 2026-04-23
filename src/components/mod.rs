use tui_realm_stdlib::components::{Input, List, Phantom};
use tuirealm::component::{AppComponent, Component};

use tuirealm::command::{
    Cmd,
    CmdResult::{self, Changed},
    Direction, Position,
};
use tuirealm::event::{Event, Key, KeyEvent, KeyModifiers};
use tuirealm::props::{AttrValue, Attribute, HorizontalAlignment, LineStatic, Title};
use tuirealm::props::{BorderType, Borders, Color, InputType, Style, Table, TableBuilder};
use tuirealm::ratatui::style::Stylize;
use tuirealm::ratatui::text::Line;

use crate::{
    AppEvent,
    Msg::{self, NoteSelected},
    backend::{Note, Todo},
};

#[derive(Component, Default)]
pub struct PhantomListener {
    component: Phantom,
}

impl AppComponent<Msg, AppEvent> for PhantomListener {
    fn on(&mut self, ev: &Event<AppEvent>) -> Option<Msg> {
        let _ = match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => return Some(Msg::AppClose),
            Event::User(AppEvent::ErrorInitialized) => return Some(Msg::AppClose),
            _ => CmdResult::NoChange,
        };
        Some(Msg::None)
    }
}
#[derive(Component)]
pub struct NoteList {
    component: List,
}

impl Default for NoteList {
    fn default() -> Self {
        Self {
            component: List::default()
                .title(
                    Into::<Title>::into("Note List")
                        .alignment(HorizontalAlignment::Left),
                )
                .highlight_style(Style::from(Color::LightYellow))
                .highlight_str("👉")
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

impl AppComponent<Msg, AppEvent> for NoteList {
    fn on(&mut self, ev: &Event<AppEvent>) -> Option<Msg> {
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
                    AttrValue::Table(Self::build_table_note(data)),
                );
                Some(NoteSelected(
                    self.component.state().unwrap_single().unwrap_usize(),
                ))
            }
            _ => Some(Msg::None),
        }
    }
}

impl NoteList {
    fn maybe_scroll_note_list(&mut self, ev: &Event<AppEvent>) -> Option<Msg> {
        if let Changed(state) = maybe_scroll_list(&mut self.component, ev) {
            return Some(NoteSelected(state.unwrap_single().unwrap_usize()));
        }
        None
    }

    pub fn build_table_note(notes: &Vec<Note>) -> Table {
        let mut table = TableBuilder::default();

        if notes.is_empty() {
            return table.build();
        }

        notes.iter().enumerate().for_each(|(index, note)| {
            let index_str = format!("{:03}", index + 1);

            let row = table
                .add_col(LineStatic::from(index_str).style(Style::default().fg(Color::Cyan).italic()))
                .add_col(LineStatic::from("    "))
                .add_col(LineStatic::from(note.title().unwrap()));

            if index < notes.len() - 1 {
                row.add_row();
            }
        });

        table.build()
    }
}

#[derive(Component)]
pub struct ShortcutsLegend {
    component: tui_realm_stdlib::components::Table,
}

impl Default for ShortcutsLegend {
    fn default() -> Self {
        Self {
            component: tui_realm_stdlib::components::Table::default()
                .title(
                    Title::from("Key Bindings")
                        .alignment(HorizontalAlignment::Left),
                )
                .scroll(false)
                .borders(Borders::default().modifiers(BorderType::Double))
                .table(
                    TableBuilder::default()
                        .add_col(LineStatic::from(" ESC").bold())
                        .add_col(LineStatic::from("  "))
                        .add_col(LineStatic::from("Quit the application"))
                        .add_col(LineStatic::from("           "))
                        .add_col(LineStatic::from(" A").bold())
                        .add_col(LineStatic::from("  "))
                        .add_col(LineStatic::from("Add note/item"))
                        .add_row()
                        .add_col(LineStatic::from(" TAB").bold())
                        .add_col(LineStatic::from("  "))
                        .add_col(LineStatic::from("Switch focus"))
                        .add_col(LineStatic::from("                   "))
                        .add_col(LineStatic::from(" E").bold())
                        .add_col(LineStatic::from("  "))
                        .add_col(LineStatic::from("Edit note/item"))
                        .add_row()
                        .add_col(LineStatic::from(" SPC").bold())
                        .add_col(LineStatic::from("  "))
                        .add_col(LineStatic::from("Cycle between item status"))
                        .add_col(LineStatic::from("      "))
                        .add_col(LineStatic::from(" D").bold())
                        .add_col(LineStatic::from("  "))
                        .add_col(LineStatic::from("Delete note/item"))
                        .build(),
                ),
        }
    }
}

impl AppComponent<Msg, AppEvent> for ShortcutsLegend {
    fn on(&mut self, _ev: &Event<AppEvent>) -> Option<Msg> {
        Some(Msg::None)
    }
}

#[derive(Component)]
pub struct TodoList {
    component: List,
}

impl Default for TodoList {
    fn default() -> Self {
        Self {
            component: List::default()
                .title(
                    Into::<Title>::into("Item List")
                        .alignment(HorizontalAlignment::Left),
                )
                .highlight_style(Style::from(Color::LightYellow))
                .highlight_str("👉")
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

impl AppComponent<Msg, AppEvent> for TodoList {
    fn on(&mut self, ev: &Event<AppEvent>) -> Option<Msg> {
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
            Event::Keyboard(KeyEvent {
                code: Key::Char(' '),
                ..
            }) => Some(Msg::SwitchTodoStatus),
            Event::Keyboard(KeyEvent { code: _, .. }) => self.maybe_scroll_todo_list(ev),
            _ => Some(Msg::None),
        }
    }
}

impl TodoList {
    fn maybe_scroll_todo_list(&mut self, ev: &Event<AppEvent>) -> Option<Msg> {
        if let Changed(state) = maybe_scroll_list(&mut self.component, ev) {
            return Some(Msg::TodoSelected(state.unwrap_single().unwrap_usize()));
        }
        None
    }

    pub fn build_table_todo(todos: Vec<Todo>) -> Table {
        let mut table = TableBuilder::default();

        if todos.is_empty() {
            return table.build();
        }
        
        todos.iter().enumerate().for_each(|(index, todo)| {
            let (done, space) = match todo.done().unwrap() {
                Some(true) => ("✔️", "  "),
                Some(false) => ("❌", " "),
                None => ("❓", " "),
            };

            let description = todo.description().unwrap();
            let row = table
                .add_col(Line::from(done))
                .add_col(Line::from(space))
                .add_col(Line::from(description));

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
#[derive(Component)]
pub struct EditPopup {
    component: Input,
    edit_type: EditPopupType,
}

impl AppComponent<Msg, AppEvent> for EditPopup {
    fn on(&mut self, ev: &Event<AppEvent>) -> Option<Msg> {
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
            }) => self.perform(Cmd::Type(*ch)),

            _ => CmdResult::NoChange,
        };
        match ev {
            Event::Keyboard(KeyEvent { code: Key::Esc, .. }) => match self.edit_type {
                EditPopupType::Note => Some(Msg::CloseEditNote(None)),
                EditPopupType::Todo => Some(Msg::CloseEditTodo(None)),
            },
            Event::Keyboard(KeyEvent {
                code: Key::Enter, ..
            }) => {
                let data = self.component.state().unwrap_single().unwrap_string();
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
        let popup_title = title.to_string();
        EditPopup {
            component: Input::default()
                .borders(
                    Borders::default()
                        .modifiers(BorderType::Rounded)
                        .color(Color::LightYellow),
                )
                .foreground(Color::LightYellow)
                .input_type(InputType::Text)
                .title(Title::from(LineStatic::from(popup_title.clone())).alignment(HorizontalAlignment::Left))
                .value(data)
                .invalid_style(Style::default().fg(Color::Red)),
            edit_type,
        }
    }
}

fn maybe_scroll_list(list: &mut List, ev: &Event<AppEvent>) -> CmdResult {
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
        _ => CmdResult::NoChange,
    }
}
