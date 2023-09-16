use std::{
    io,
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    listener::{ListenerResult, Poll},
    props::{PropPayload, PropValue},
    terminal::TerminalBridge,
    tui::{
        layout::{Constraint, Direction, Layout},
        prelude::Rect,
        widgets::Clear,
    },
    Application, AttrValue, Attribute, Event, EventListenerCfg, PollStrategy, Sub, SubClause,
    SubEventClause, Update,
};

use crate::{
    backend::{NotesWall, NotesWallBuilder},
    components::{EditPopup, EditPopupType, NoteList, PhantomListener, ShortcutsLegend, TodoList},
    AppEvent, Id, Msg,
};

type SharedWall = Arc<RwLock<NotesWall>>;

pub struct Model {
    quit: bool,   // Becomes true when the user presses <ESC>
    redraw: bool, // Tells whether to refresh the UI; performance optimization
    text_edit_popup_open: bool,
    selected_note_index: usize,
    selected_todo_index: usize,
    notes_wall: SharedWall,
    terminal: TerminalBridge,
    app: Application<Id, Msg, AppEvent>,
}

impl Model {
    pub fn new(path: PathBuf) -> Self {
        let quit = false;
        let redraw = true;
        let text_edit_popup_open = false;
        let selected_note_index = 0;
        let selected_todo_index = 0;
        let notes_wall = Arc::new(RwLock::new(
            NotesWallBuilder::default()
                .folder_path(path)
                .build()
                .unwrap(),
        ));
        let mut terminal = TerminalBridge::new().expect("Cannot create terminal bridge");
        let _ = terminal.enable_raw_mode();
        let _ = terminal.enter_alternate_screen();
        let mut app: Application<Id, Msg, AppEvent> = Application::init(
            EventListenerCfg::default()
                .default_input_listener(Duration::from_millis(10))
                .port(
                    Box::new(NotesProvider::new(notes_wall.clone())),
                    Duration::from_millis(100),
                ),
        );
        assert!(app
            .mount(Id::NoteList, Box::<NoteList>::default(), vec![])
            .is_ok());
        assert!(app
            .mount(Id::InfoBox, Box::<ShortcutsLegend>::default(), vec![])
            .is_ok());
        assert!(app
            .mount(Id::TodoList, Box::<TodoList>::default(), vec![])
            .is_ok());
        assert!(app
            .mount(
                Id::PhantomListener,
                Box::<PhantomListener>::default(),
                vec![
                    Sub::new(
                        SubEventClause::Keyboard(KeyEvent {
                            code: Key::Esc,
                            modifiers: KeyModifiers::NONE
                        }),
                        SubClause::Always
                    ),
                    Sub::new(
                        SubEventClause::User(AppEvent::ErrorInitialized),
                        SubClause::Always
                    )
                ]
            )
            .is_ok());

        // We need to give focus to input then
        assert!(app.active(&Id::NoteList).is_ok());

        Self {
            quit,
            redraw,
            text_edit_popup_open,
            selected_note_index,
            selected_todo_index,
            terminal,
            app,
            notes_wall,
        }
    }

    pub fn main_loop(&mut self) {
        while !self.quit {
            // Tick
            if let Ok(messages) = self.app.tick(PollStrategy::Once) {
                messages.iter().map(Some).for_each(|msg| {
                    let mut msg = msg.cloned();
                    while msg.is_some() {
                        msg = self.update(msg);
                    }
                });
            }

            // Redraw
            if self.redraw {
                self.view();
                self.redraw = false;
            }
        }
        let _ = self.terminal.leave_alternate_screen();
        let _ = self.terminal.disable_raw_mode();
        let _ = self.terminal.clear_screen();
    }

    fn view(&mut self) {
        let _ = self.terminal.raw_mut().draw(|f| {
            // Prepare chunks
            let main_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(f.size());

            let sub_chunk = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(80), Constraint::Percentage(20)])
                .split(main_chunks[0]);

            self.app.view(&Id::NoteList, f, sub_chunk[0]);
            self.app.view(&Id::InfoBox, f, sub_chunk[1]);
            self.app.view(&Id::TodoList, f, main_chunks[1]);

            if self.text_edit_popup_open {
                let popup = Self::draw_area_in_absolute(f.size(), 30, 3);
                f.render_widget(Clear, popup);
                self.app.view(&Id::EditPopup, f, popup);
            }
        });
    }

    fn draw_area_in_absolute(parent: Rect, width: u16, height: u16) -> Rect {
        let new_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length((parent.height - height) / 2),
                    Constraint::Length(height),
                    Constraint::Length((parent.height - height) / 2),
                ]
                .as_ref(),
            )
            .split(parent);
        Layout::default()
            .direction(Direction::Horizontal)
            .constraints(
                [
                    Constraint::Length((parent.width - width) / 2),
                    Constraint::Length(width),
                    Constraint::Length((parent.width - width) / 2),
                ]
                .as_ref(),
            )
            .split(new_area[1])[1]
    }
}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        self.redraw = true;
        match msg.unwrap_or(Msg::None) {
            Msg::AppClose => {
                if !self.text_edit_popup_open {
                    self.quit = true;
                }
                None
            }
            Msg::CloseEditNote(data) => self.update_note_title(data),
            Msg::CloseEditTodo(data) => self.update_note_todo(data),
            Msg::None => None,
            Msg::NoteSelected(index) => {
                self.selected_note_index = index;
                self.reload_todo_list()
            }
            Msg::TodoSelected(index) => {
                self.selected_todo_index = index;
                None
            }
            Msg::NoteListBlur => {
                assert!(self.app.active(&Id::TodoList).is_ok());
                None
            }
            Msg::TodoListBlur => {
                assert!(self.app.active(&Id::NoteList).is_ok());
                None
            }
            Msg::EditNote => self.prepare_note_edit_popup(),
            Msg::AddNote => self.add_note(),
            Msg::RemoveNote => self.remove_note(),
            Msg::ReloadNoteList => self.reload_note_list(),
            Msg::ReloadTodoList => self.reload_todo_list(),
            Msg::EditTodo => self.prepare_todo_edit_popup(),
            Msg::AddTodo => self.add_todo(),
            Msg::RemoveTodo => self.remove_todo(),
            Msg::SwitchTodoStatus => self.switch_todo_status(),
        }
    }
}

impl Model {
    fn switch_todo_status(&mut self) -> Option<Msg> {
        let guard = self.notes_wall.write().unwrap();
        if let Some(note) = guard.get_notes().get_mut(self.selected_note_index) {
            if let Some(todo) = note.todos().get(self.selected_todo_index) {
                let new_done = match todo.done().unwrap() {
                    Some(true) => Some(false),
                    Some(false) => None,
                    None => Some(true),
                };
                assert!(todo.set_done(new_done).is_ok());
                assert!(note.save().is_ok());
            }
        }
        Some(Msg::ReloadTodoList)
    }

    fn remove_todo(&mut self) -> Option<Msg> {
        let guard = self.notes_wall.write().unwrap();
        if let Some(note) = guard.get_notes().get_mut(self.selected_note_index) {
            if let Some(todo) = note.todos().get(self.selected_todo_index) {
                assert!(note.remove_todo(todo).is_ok());
                assert!(note.save().is_ok());
                self.selected_todo_index = 0;
            }
        }
        Some(Msg::ReloadTodoList)
    }

    fn remove_note(&mut self) -> Option<Msg> {
        let mut guard = self.notes_wall.write().unwrap();
        if let Some(note) = guard.get_notes().get(self.selected_note_index) {
            assert!(guard.remove_note(note).is_ok());
            self.selected_note_index = 0;
        }
        Some(Msg::ReloadNoteList)
    }

    fn add_note(&mut self) -> Option<Msg> {
        self.selected_note_index = self.notes_wall.read().unwrap().get_notes().len();
        self.notes_wall.write().unwrap().create_note();
        Some(Msg::EditNote)
    }

    fn add_todo(&mut self) -> Option<Msg> {
        let guard = self.notes_wall.write().unwrap();
        if let Some(note) = guard.get_notes().get_mut(self.selected_note_index) {
            if let Ok(_) = note.create_todo() {
                self.selected_todo_index = note.todos().len() - 1;
                return Some(Msg::EditTodo);
            }
        }
        None
    }

    fn update_note_todo(&mut self, description: Option<String>) -> Option<Msg> {
        self.text_edit_popup_open = false;
        assert!(self.app.umount(&Id::EditPopup).is_ok());
        if let Some(description) = description {
            if let Some(note) = self
                .notes_wall
                .read()
                .unwrap()
                .get_notes()
                .get(self.selected_note_index)
            {
                let _ = note.todos()[self.selected_todo_index].set_description(&description);
                assert!(note.save().is_ok());
            }
        }
        Some(Msg::ReloadTodoList)
    }

    fn update_note_title(&mut self, title: Option<String>) -> Option<Msg> {
        self.text_edit_popup_open = false;
        assert!(self.app.umount(&Id::EditPopup).is_ok());

        if let Some(title) = title {
            if let Some(note) = self
                .notes_wall
                .read()
                .unwrap()
                .get_notes()
                .get(self.selected_note_index)
            {
                let _ = note.set_title(&title);
                assert!(note.save().is_ok());
            }
        }

        Some(Msg::ReloadNoteList)
    }

    fn reload_note_list(&mut self) -> Option<Msg> {
        assert!(self
            .app
            .attr(
                &Id::NoteList,
                Attribute::Content,
                AttrValue::Table(NoteList::build_table_note(
                    self.notes_wall.read().unwrap().get_notes()
                ))
            )
            .is_ok());

        assert!(self
            .app
            .attr(
                &Id::NoteList,
                Attribute::Value,
                AttrValue::Payload(PropPayload::One(PropValue::Usize(self.selected_note_index)))
            )
            .is_ok());

        self.selected_todo_index = 0;
        Some(Msg::ReloadTodoList)
    }

    fn prepare_note_edit_popup(&mut self) -> Option<Msg> {
        if let Some(note) = self
            .notes_wall
            .read()
            .unwrap()
            .get_notes()
            .get(self.selected_note_index)
        {
            self.text_edit_popup_open = true;
            assert!(self
                .app
                .remount(
                    Id::EditPopup,
                    Box::new(EditPopup::new(
                        &note.title().unwrap(),
                        "Title",
                        EditPopupType::Note
                    )),
                    vec![]
                )
                .is_ok());
            assert!(self.app.active(&Id::EditPopup).is_ok());
        }
        None
    }

    fn prepare_todo_edit_popup(&mut self) -> Option<Msg> {
        if let Some(note) = self
            .notes_wall
            .read()
            .unwrap()
            .get_notes()
            .get(self.selected_note_index)
        {
            if let Some(todo) = note.todos().get(self.selected_todo_index) {
                self.text_edit_popup_open = true;
                assert!(self
                    .app
                    .remount(
                        Id::EditPopup,
                        Box::new(EditPopup::new(
                            &todo.description().unwrap(),
                            "ToDo",
                            EditPopupType::Todo
                        )),
                        vec![]
                    )
                    .is_ok());
                assert!(self.app.active(&Id::EditPopup).is_ok());
            }
        }
        None
    }

    fn reload_todo_list(&mut self) -> Option<Msg> {
        match self
            .notes_wall
            .read()
            .unwrap()
            .get_notes()
            .get(self.selected_note_index)
        {
            Some(note) => {
                assert!(self
                    .app
                    .attr(
                        &Id::TodoList,
                        Attribute::Content,
                        AttrValue::Table(TodoList::build_table_todo(note.todos()))
                    )
                    .is_ok());

                assert!(self
                    .app
                    .attr(
                        &Id::TodoList,
                        Attribute::Value,
                        AttrValue::Payload(PropPayload::One(PropValue::Usize(
                            self.selected_todo_index
                        )))
                    )
                    .is_ok());
            }
            None => assert!(self
                .app
                .attr(
                    &Id::TodoList,
                    Attribute::Content,
                    AttrValue::Table(TodoList::build_table_todo(vec![]))
                )
                .is_ok()),
        }
        None
    }
}

struct NotesProvider {
    wall: SharedWall,
    init: Option<io::Result<()>>,
}

impl NotesProvider {
    fn new(wall: SharedWall) -> Self {
        let init = Some(wall.write().unwrap().init());

        NotesProvider { wall, init }
    }
}

impl Poll<AppEvent> for NotesProvider {
    fn poll(&mut self) -> ListenerResult<Option<Event<AppEvent>>> {
        if let Some(result) = self.init.take() {
            return match result {
                Ok(_) => Ok(Some(Event::User(AppEvent::NoteLoaded(
                    self.wall.read().unwrap().get_notes(),
                )))),
                Err(_) => Ok(Some(Event::User(AppEvent::ErrorInitialized))),
            };
        };

        Ok(None)
    }
}
