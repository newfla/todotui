use std::time::Duration;

use tuirealm::{
    terminal::TerminalBridge,
    tui::layout::{Constraint, Direction, Layout},
    Application, EventListenerCfg, NoUserEvent, PollStrategy, Update,
};

use crate::{components::RootContainer, Id, Msg};

pub struct Model {
    quit: bool,   // Becomes true when the user presses <ESC>
    redraw: bool, // Tells whether to refresh the UI; performance optimization
    terminal: TerminalBridge,
    app: Application<Id, Msg, NoUserEvent>,
}

impl Default for Model {
    fn default() -> Self {
        let quit = false;
        let redraw = true;
        let mut terminal = TerminalBridge::new().expect("Cannot create terminal bridge");
        let _ = terminal.enable_raw_mode();
        let _ = terminal.enter_alternate_screen();
        let mut app: Application<Id, Msg, NoUserEvent> = Application::init(
            EventListenerCfg::default().default_input_listener(Duration::from_millis(10)),
        );
        assert!(app
            .mount(
                Id::RootContainer,
                Box::new(RootContainer::default()),
                vec![]
            )
            .is_ok());

         // We need to give focus to input then
         assert!(app.active(&Id::RootContainer).is_ok());

        Self {
            quit,
            redraw,
            terminal,
            app,
        }
    }
}

impl Model {
    pub fn main_loop(&mut self) {
        while !self.quit {
            // Tick
            if let Ok(messages) = self.app.tick(PollStrategy::Once) {
                messages.iter().map(|m| Some(m)).for_each(|msg| {
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
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Length(100)].as_ref())
                .split(f.size());
            self.app.view(&Id::RootContainer, f, chunks[0]);
        });
    }
}

impl Update<Msg> for Model {
    fn update(&mut self, msg: Option<Msg>) -> Option<Msg> {
        match msg.unwrap_or(Msg::None) {
            Msg::AppClose => {
                self.quit = true;
                None
            }
            Msg::None => None,
        }
    }
}
