use std::{
    io,
    path::Path,
    sync::{Arc, RwLock},
    time::Duration,
};

use tuirealm::{
    event::{Key, KeyEvent, KeyModifiers},
    listener::{ListenerResult, Poll},
    terminal::TerminalBridge,
    tui::{
        layout::{Constraint, Direction, Layout},
        prelude::Rect,
        widgets::Clear,
    },
    Application, Attribute, Event, EventListenerCfg, PollStrategy, Sub, SubClause, SubEventClause,
    Update,
};

use crate::{
    backend::{NotesWall, NotesWallBuilder},
    components::{EditPopup, NoteList, PhantomListener, ShortcutsLegend, TodoList},
    AppEvent, Id, Msg,
};

type SharedWall = Arc<RwLock<NotesWall>>;

pub struct Model {
    quit: bool,   // Becomes true when the user presses <ESC>
    redraw: bool, // Tells whether to refresh the UI; performance optimization
    text_edit_popup: bool,
    notes_wall: SharedWall,
    terminal: TerminalBridge,
    app: Application<Id, Msg, AppEvent>,
}

impl Default for Model {
    fn default() -> Self {
        let quit = false;
        let redraw = true;
        let text_edit_popup = false;
        let notes_wall = Arc::new(RwLock::new(
            NotesWallBuilder::default()
                .folder_path(Path::new("/tmp/test_todotui").to_path_buf())
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
                        SubEventClause::User(AppEvent::ErrorInitiliazed),
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
            text_edit_popup,
            terminal,
            app,
            notes_wall,
        }
    }
}

impl Model {
    pub fn main_loop(&mut self) {
        while !self.quit {
            // Tick
            if let Ok(messages) = self.app.tick(PollStrategy::Once) {
                messages.iter().map(Some).for_each(|msg| {
                    let mut msg = msg.copied();
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

            if self.text_edit_popup {
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
                self.quit = true;
                None
            }
            Msg::None => None,
            Msg::NoteSelected(index) => self.update_todo_list(index),
            Msg::TodoSelected(_) => None,
            Msg::NoteListBlur => {
                assert!(self.app.active(&Id::TodoList).is_ok());
                None
            }
            Msg::TodoListBlur => {
                assert!(self.app.active(&Id::NoteList).is_ok());
                None
            }
            Msg::EditNote(index) => self.prepare_note_edit_popup(index),
        }
    }
}

impl Model {
    
    fn prepare_note_edit_popup(&mut self, index: usize) -> Option<Msg> {
        if let Some(note) = self.notes_wall.read().unwrap().get_notes().get(index) {
            self.text_edit_popup = true;
            assert!(self
                .app
                .remount(Id::EditPopup, Box::new(EditPopup::new(&note.title().unwrap())), vec![])
                .is_ok());
            assert!(self.app.active(&Id::EditPopup).is_ok());
        }
        None
    }

    fn update_todo_list(&mut self, index: usize) -> Option<Msg> {
        assert!(self
            .app
            .remount(Id::TodoList, Box::<TodoList>::default(), vec![])
            .is_ok());
        if let Some(note) = self.notes_wall.read().unwrap().get_notes().get(index) {
            let todos = note.todos();
            if todos.is_empty() {
                return None;
            }
            assert!(self
                .app
                .attr(
                    &Id::TodoList,
                    Attribute::Content,
                    tuirealm::AttrValue::Table(TodoList::build_table_todo(note.todos())),
                )
                .is_ok());
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
            match result {
                Ok(_) => {
                    return Ok(Some(Event::User(AppEvent::NoteLoaded(
                        self.wall.read().unwrap().get_notes(),
                    ))))
                }
                Err(_) => return Ok(Some(Event::User(AppEvent::ErrorInitiliazed))),
            }
        };

        Ok(None)
    }
}
