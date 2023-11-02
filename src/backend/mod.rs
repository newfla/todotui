use std::{
    fs::{read, read_dir, remove_file, write},
    hash::Hash,
    ops::Deref,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use anyhow::{bail, ensure, Context, Result};
use chrono::Utc;
use derive_builder::Builder;
use postcard::{from_bytes, to_stdvec};
use serde::{Deserialize, Serialize};

static DATE_FORMAT: &str = "%d_%m_%Y_%H:%M_%6f";
static FILE_EXTENSION: &str = "post";
static POISONED: &str = "Poisoned mutex";
static EMPTY_NOTE: &str = "Note is empty";
static FAILED_SERIALIZATION: &str = "Failed to serialize";
static FAILED_REMOVE: &str = "Failed to remove";

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
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

impl PartialOrd for Todo {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            self.0
                .read()
                .unwrap()
                .deref()
                .cmp(other.0.read().unwrap().deref()),
        )
    }
}

impl Todo {
    pub fn done(&self) -> Result<Option<bool>> {
        let lock = self.0.read();
        ensure!(lock.is_ok(), POISONED);
        Ok(lock.unwrap().done)
    }

    pub fn set_done(&self, done: Option<bool>) -> Result<()> {
        let lock = self.0.write();
        ensure!(lock.is_ok(), POISONED);
        lock.unwrap().done = done;
        Ok(())
    }

    pub fn description(&self) -> Result<String> {
        let lock = self.0.read();
        ensure!(lock.is_ok(), POISONED);
        Ok(lock.unwrap().description.clone())
    }

    pub fn set_description(&self, description: &str) -> Result<()> {
        let lock = self.0.write();
        ensure!(lock.is_ok(), POISONED);
        lock.unwrap().description = description.to_string();
        Ok(())
    }
}

#[derive(Eq, Clone, Deserialize, Serialize, Debug, PartialOrd)]
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

    fn remove_todo(&mut self, todo: &Todo) {
        let index = self.todos.iter().position(|e| e == todo);
        if let Some(index) = index {
            self.todos.remove(index);
        }
    }
}

#[derive(Debug, Eq)]
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

impl PartialOrd for PersistenceInternalNote {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
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

impl PartialOrd for Note {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0
            .read()
            .unwrap()
            .deref()
            .partial_cmp(other.0.read().unwrap().deref())
    }
}

impl Hash for Note {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.read().unwrap().path.hash(state);
    }
}

impl Note {
    fn set_path(&self, path: PathBuf) -> Result<()> {
        let lock = self.0.write();
        ensure!(lock.is_ok(), POISONED);
        lock.unwrap().path = path;
        Ok(())
    }

    pub fn set_title(&self, title: &str) -> Result<()> {
        let lock = self.0.write();
        ensure!(lock.is_ok(), POISONED);
        match lock.unwrap().note.as_mut() {
            Some(note) => {
                note.title = title.to_string();
                Ok(())
            }
            None => bail!(EMPTY_NOTE),
        }
    }

    pub fn title(&self) -> Result<String> {
        match self.title_internal() {
            Ok(title) => match title.is_empty() {
                true => Ok(self.created().unwrap()),
                false => Ok(title),
            },
            Err(_) => Ok(self.created().unwrap()),
        }
    }

    fn created(&self) -> Result<String> {
        let lock = self.0.read();
        ensure!(lock.is_ok(), POISONED);
        match &lock.unwrap().note {
            Some(data) => Ok(data.created.clone()),
            None => Ok("".to_string()),
        }
    }

    fn title_internal(&self) -> Result<String> {
        let lock = self.0.read();
        ensure!(lock.is_ok(), POISONED);
        match &lock.unwrap().note {
            Some(data) => Ok(data.title.clone()),
            None => bail!(EMPTY_NOTE),
        }
    }

    pub fn create_todo(&mut self) -> Result<Todo> {
        let lock = self.0.write();
        ensure!(lock.is_ok(), POISONED);
        match lock.unwrap().note.as_mut() {
            Some(note) => {
                let todo = Todo::default();
                note.add_todo(todo.clone());
                Ok(todo)
            }
            None => bail!(EMPTY_NOTE),
        }
    }

    pub fn remove_todo(&mut self, todo: &Todo) -> Result<()> {
        let lock = self.0.write();
        ensure!(lock.is_ok(), POISONED);
        match lock.unwrap().note.as_mut() {
            Some(note) => {
                note.remove_todo(todo);
                Ok(())
            }
            None => bail!(EMPTY_NOTE),
        }
    }

    pub fn todos(&self) -> Vec<Todo> {
        self.0.read().map_or(Vec::new(), |data| match &data.note {
            Some(data) => data.todos.to_vec(),
            None => Vec::new(),
        })
    }

    fn load(&self) -> bool {
        let path = self.0.read().unwrap().path.clone();
        read(path)
            .map(|data| from_bytes::<InternalNote>(&data))
            .map_or(false, |note| match note {
                Ok(note) => match self.0.write() {
                    Ok(mut data) => {
                        data.note = Some(note);
                        true
                    }
                    Err(_) => false,
                },
                Err(_) => false,
            })
    }

    pub fn save(&self) -> Result<()> {
        let lock = self.0.read().unwrap();

        lock.note
            .as_ref()
            .map_or(Ok(()), |note| match to_stdvec(&note) {
                std::result::Result::Ok(data) => {
                    write(lock.path.clone(), data).context(FAILED_SERIALIZATION)
                }
                Err(_) => bail!(FAILED_SERIALIZATION),
            })
    }
}

#[derive(Builder, Default)]
pub struct NotesWall {
    folder_path: PathBuf,
    #[builder(setter(skip))]
    notes: Vec<Note>,
}

impl NotesWall {
    pub fn init(&mut self) -> Result<()> {
        self.notes = read_dir(self.folder_path.as_path())?
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
            .filter(|note| note.load())
            .collect();
        Ok(())
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

    pub fn remove_note(&mut self, note: &Note) -> Result<()> {
        self.notes
            .iter()
            .position(|e| e == note)
            .map_or(Ok(()), |index| {
                self.notes.remove(index);
                let path = {
                    let data_guard = note.0.read().unwrap();
                    data_guard.path.clone()
                };
                remove_file(path.as_path()).context(FAILED_REMOVE)
            })
    }

    fn save_all(&self) -> Result<()> {
        let mut status = Ok(());
        for e in self.notes.iter() {
            let result = e.save();
            status = status.and(result);
        }
        status
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::Path};

    use crate::backend::NotesWallBuilder;

    static TEST_FOLDER_PATH: &str = "/tmp/test_todotui";

    fn init_test_folder() {
        cleanup_test_folder();
        fs::create_dir_all(TEST_FOLDER_PATH).unwrap();
        assert!(fs::metadata(TEST_FOLDER_PATH).unwrap().is_dir());
    }

    fn cleanup_test_folder() {
        let _ = fs::remove_dir_all(TEST_FOLDER_PATH);
    }

    #[test]
    fn standard_test() {
        //Create work dir
        init_test_folder();

        let mut wall_1 = NotesWallBuilder::default()
            .folder_path(Path::new(TEST_FOLDER_PATH).to_path_buf())
            .build()
            .unwrap();
        let wall_2 = NotesWallBuilder::default()
            .folder_path(Path::new(TEST_FOLDER_PATH).to_path_buf())
            .build()
            .unwrap();

        //This will load an empty wall due to empty work dir
        assert!(wall_1.init().is_ok());
        assert_eq!(wall_1.get_notes().len(), 0);

        let mut note_1 = wall_1.create_note();
        let mut note_2 = wall_1.create_note();
        let _ = wall_1.create_note();

        assert!(note_1.set_title("note_1").is_ok());
        assert!(note_2.set_title("note_2").is_ok());

        let todo_1 = note_1.create_todo().unwrap();
        let todo_2 = note_1.create_todo().unwrap();
        let todo_3 = note_2.create_todo().unwrap();
        let todo_4 = note_2.create_todo().unwrap();
        let todo_5 = note_2.create_todo().unwrap();

        assert!(todo_1.set_description("desc1").is_ok());
        assert!(todo_1.set_done(Some(true)).is_ok());
        assert!(todo_2.set_description("desc2").is_ok());
        assert!(todo_2.set_done(Some(true)).is_ok());
        assert!(todo_3.set_description("desc3").is_ok());
        assert!(todo_3.set_done(Some(false)).is_ok());
        assert!(todo_4.set_description("desc4").is_ok());
        assert!(todo_4.set_done(None).is_ok());
        assert!(todo_5.set_description("desc5").is_ok());
        assert!(todo_5.set_done(Some(true)).is_ok());

        //wall_1 has two notes attached to it
        let binding_wall_1 = wall_1.get_notes();
        let mut notes_wall_1 = binding_wall_1.iter();
        assert_eq!(note_1, notes_wall_1.next().unwrap().to_owned());
        assert_eq!(note_2, notes_wall_1.next().unwrap().to_owned());

        assert!(wall_1.save_all().is_ok());
    }
}
