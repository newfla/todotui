mod backend;
mod components;
pub mod model;

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
enum Id {
    RootContainer,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Msg {
    AppClose,
    None,
}
