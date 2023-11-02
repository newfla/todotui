#![doc = include_str!("../README.md")]
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

#[derive(Debug, PartialEq, Clone)]
enum Msg {
    AppClose,
    None,
    NoteSelected(usize),
    TodoSelected(usize),
    EditNote,
    AddNote,
    RemoveNote,
    CloseEditNote(Option<String>),
    CloseEditTodo(Option<String>),
    NoteListBlur,
    TodoListBlur,
    ReloadNoteList,
    ReloadTodoList,
    EditTodo,
    AddTodo,
    RemoveTodo,
    SwitchTodoStatus,
}

#[derive(PartialEq, Eq, Clone, PartialOrd)]
enum AppEvent {
    ErrorInitialized,
    NoteLoaded(Vec<Note>),
}
