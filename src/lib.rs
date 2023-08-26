use backend::Note;

mod backend;
mod components;
pub mod model;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
enum Id {
    PhantomListener,
    NoteList,
    TodoList,
    InfoBox,
    EditPopup,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Msg {
    AppClose,
    None,
    NoteSelected(usize),
    TodoSelected(usize),
    EditNote(usize),
    NoteListBlur,
    TodoListBlur,
}

#[derive(PartialEq, Eq, Clone, PartialOrd)]
pub enum AppEvent {
    ErrorInitiliazed,
    NoteLoaded(Vec<Note>),
}

