use std::{
    hash::Hash,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock}, io::Error,
};

use chrono::Utc;
use derive_builder::Builder;
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};
use tokio::{fs::{read, read_dir, remove_file, write}, io};
use tokio_stream::{wrappers::ReadDirStream, StreamExt};

static DATE_FORMAT: &str = "%d_%m_%Y_%H:%M_%6f";
static FILE_EXTENSION: &str = "post";
static POISONED: &str = "Poisoned mutex";
static EMPTY_NOTE: &str = "Note is empty";
static FAILED_SERIALIZATION: &str = "Failed to serialize";

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Default)]
struct InternalTodo {
    done: Option<bool>,
    description: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct Todo(Arc<RwLock<InternalTodo>>);

impl PartialEq for Todo {
    fn eq(&self, other: &Self) -> bool {
        self.0.read().unwrap().deref() == other.0.read().unwrap().deref()
    }
}

impl Eq for Todo {}

impl Todo {
    pub fn get_done(&self) -> Result<Option<bool>, &'static str> {
        match self.0.read() {
            Ok(data) => Ok(data.done),
            Err(_) => Err(POISONED),
        }
    }

    pub fn set_done(&self, done: Option<bool>) -> Result<(), &'static str> {
        match self.0.write() {
            Ok(mut data) => {
                data.done = done;
                Ok(())
            }
            Err(_) => Err(POISONED),
        }
    }

    pub fn get_description(&self) -> Result<String, &'static str> {
        match self.0.read() {
            Ok(data) => Ok(data.description.clone()),
            Err(_) => Err(POISONED),
        }
    }

    pub fn set_description(&self, description: &str) -> Result<(), &'static str> {
        match self.0.write() {
            Ok(mut data) => {
                data.description = description.to_string();
                Ok(())
            }
            Err(_) => Err(POISONED),
        }
    }
}

#[derive(Eq, Clone, Deserialize, Serialize, Debug)]
struct InternalNote {
    title: String,
    created: String,
    todos: Vec<Todo>,
}

impl Default for InternalNote {
    fn default() -> Self {
        let created = format!("{}", Utc::now().format(DATE_FORMAT));
        Self {
            title: Default::default(),
            created,
            todos: Default::default(),
        }
    }
}

impl PartialEq for InternalNote {
    fn eq(&self, other: &Self) -> bool {
        self.title == other.title && self.created == other.created
    }
}

impl Hash for InternalNote {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.title.hash(state);
        self.created.hash(state);
    }
}

impl InternalNote {
    fn add_todo(&mut self, todo: Todo) {
        self.todos.push(todo)
    }

    fn remove_todo(&mut self, todo: Todo) {
        let index = self.todos.iter().position(|e| e == &todo);
        if let Some(index) = index {
            self.todos.remove(index);
        }
    }
}

#[derive(Eq, Debug)]
struct PersistenceInternalNote {
    path: PathBuf,
    note: Option<InternalNote>,
}

impl Default for PersistenceInternalNote {
    fn default() -> Self {
        Self {
            path: Default::default(),
            note: Some(Default::default()),
        }
    }
}

impl PartialEq for PersistenceInternalNote {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Hash for PersistenceInternalNote {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.path.hash(state);
    }
}

#[derive(Default, Clone, Debug)]
pub struct Note(Arc<RwLock<PersistenceInternalNote>>);

impl PartialEq for Note {
    fn eq(&self, other: &Self) -> bool {
        self.0.read().unwrap().deref() == other.0.read().unwrap().deref()
    }
}

impl Eq for Note {}

impl Hash for Note {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.read().unwrap().path.hash(state);
    }
}

impl Note {
    fn set_path(&self, path: PathBuf) -> Result<(), &'static str> {
        match self.0.write() {
            Ok(mut data) => {
                data.path = path;
                Ok(())
            }
            Err(_) => Err(POISONED),
        }
    }

    pub fn set_title(&self, title: &str) -> Result<(), &'static str> {
        match self.0.write() {
            Ok(mut data) => match data.note.as_mut() {
                Some(note) => {
                    note.title = title.to_string();
                    Ok(())
                }
                None => Err(EMPTY_NOTE),
            },
            Err(_) => Err(POISONED),
        }
    }

    pub fn create_todo(&mut self) -> Result<Todo, &'static str> {
        match self.0.write() {
            Ok(mut data_guard) => match data_guard.note.as_mut() {
                Some(note) => {
                    let todo = Todo::default();
                    note.add_todo(todo.clone());
                    Ok(todo)
                }
                None => Err(EMPTY_NOTE),
            },
            Err(_) => Err(POISONED),
        }
    }

    pub fn remove_todo(&mut self, todo: Todo) -> Result<(), &'static str> {
        match self.0.write() {
            Ok(mut data_guard) => match data_guard.note.as_mut() {
                Some(note) => {
                    note.remove_todo(todo);
                    Ok(())
                }
                None => Err(EMPTY_NOTE),
            },
            Err(_) => Err(POISONED),
        }
    }

    pub fn get_todos(&self) -> Vec<Todo> {
        match self.0.read() {
            Ok(data_guard) => match &data_guard.note {
                Some(data) => data.todos.to_vec(),
                None => Vec::new(),
            },
            Err(_) => Vec::new(),
        }
    }

    async fn load(&self) -> bool {
        let path = self.0.read().unwrap().path.clone();
        let note = match read(path).await.map(|data| from_bytes(&data)) {
            Ok(note) => note.ok(),
            Err(_) => None,
        };

        match note {
            Some(note) => match self.0.write() {
                Ok(mut data) => {
                    data.note = Some(note);
                    true
                }
                Err(_) => false,
            },
            None => false,
        }
    }

    async fn save(&self) -> io::Result<()> {
        let (note, path) = {
            let data_guard = self.0.read().unwrap();
            (data_guard.note.clone(), data_guard.path.clone())
        };

        match note {
            Some(note) => match to_stdvec(&note) {
                Ok(data) => write(path, data).await,
                Err(_) => Err(Error::new(io::ErrorKind::Other,FAILED_SERIALIZATION)),
            },
            None => Ok(()),
        }
    }
}

#[derive(Builder, Default)]
struct NotesWall {
    folder_path: PathBuf,
    #[builder(setter(skip))]
    notes: Vec<Note>,
}

impl NotesWall {
    pub async fn init(&mut self) -> &Self {
        self.notes = match read_dir(self.folder_path.as_path()).await {
            Err(_) => Vec::default(),
            Ok(data) => {
                ReadDirStream::new(data)
                    .filter(|file| {
                        file.as_ref().is_ok_and(|f| {
                            f.file_name()
                                .to_str()
                                .unwrap()
                                .contains(&(".".to_owned() + FILE_EXTENSION))
                        })
                    })
                    .map(|path| {
                        let note = Note::default();
                        let _ = note.set_path(path.unwrap().path());
                        note
                    })
                    .collect()
                    .await
            }
        };
        self
    }

    pub fn get_notes(&self) -> Vec<Note> {
        self.notes.to_vec()
    }

    pub fn create_note(&mut self) -> Note {
        let note = Note::default();
        let mut path = self.folder_path.clone();
        path.push(
            note.0
                .read()
                .unwrap()
                .note
                .as_ref()
                .unwrap()
                .created
                .clone(),
        );
        path.set_extension(FILE_EXTENSION);
        let _ = note.set_path(path);
        self.notes.push(note.clone());
        note
    }

    pub async fn remove_note(&mut self, note: Note) -> io::Result<()> {
        let index = self.notes.iter().position(|e| e == &note);
        match index {
            Some(index) =>{
                self.notes.remove(index);
                let path = {
                    let data_guard = note.0.read().unwrap();
                    data_guard.path.clone()
                };
                remove_file(path.as_path()).await
            },
            None => Ok(()),
        }
    }

    pub async fn save_all(&self) -> io::Result<()> {
        let mut status = Ok(());
        for e in self.notes.iter() {
            let result = e.save().await;
            status = status.and(result);
        }
        status
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::NotesWallBuilder;

    static TEST_FOLDER_PATH: &str = "/tmp/test_todotui";

    fn init_test_folder() {
        cleanup_test_folder();
        fs::create_dir_all(TEST_FOLDER_PATH).unwrap();
        assert!(fs::metadata(TEST_FOLDER_PATH).unwrap().is_dir());
    }

    fn cleanup_test_folder() {
        let _ = fs::remove_dir_all(TEST_FOLDER_PATH);
    }

    #[tokio::test]
    async fn standard_test() {
        //Create work dir
        init_test_folder();

        let mut wall_1 = NotesWallBuilder::default()
            .folder_path(Path::new(TEST_FOLDER_PATH).to_path_buf())
            .build()
            .unwrap();
        let mut wall_2 = NotesWallBuilder::default()
            .folder_path(Path::new(TEST_FOLDER_PATH).to_path_buf())
            .build()
            .unwrap();

        //This will load an empty wall due to empty work dir
        wall_1.init().await;
        assert_eq!(wall_1.get_notes().len(), 0);

        let mut note_1 = wall_1.create_note();
        let mut note_2 = wall_1.create_note();

        assert!(note_1.set_title("note_1").is_ok());
        assert!(note_2.set_title("note_2").is_ok());

        let todo_1 = note_1.create_todo().unwrap();
        let todo_2 = note_1.create_todo().unwrap();
        let todo_3 = note_2.create_todo().unwrap();

        assert!(todo_1.set_description("desc1").is_ok());
        assert!(todo_1.set_done(Some(true)).is_ok());
        assert!(todo_2.set_description("desc2").is_ok());
        assert!(todo_2.set_done(Some(true)).is_ok());
        assert!(todo_3.set_description("desc3").is_ok());
        assert!(todo_3.set_done(Some(false)).is_ok());

        //wall_1 has two notes attached to it
        let binding_wall_1 = wall_1.get_notes();
        let mut notes_wall_1 = binding_wall_1.iter();
        assert_eq!(note_1, notes_wall_1.next().unwrap().to_owned());
        assert_eq!(note_2, notes_wall_1.next().unwrap().to_owned());

        assert!(wall_1.save_all().await.is_ok());

        //wall_2 points to the same wall_1 work dir
        let binding_wall_2 = wall_2.init().await.get_notes();
        let mut notes_wall_2 = binding_wall_2.iter();

        assert_eq!(note_1, notes_wall_2.next().unwrap().to_owned());
        assert_eq!(note_2, notes_wall_2.next().unwrap().to_owned());

        cleanup_test_folder();
    }
}
