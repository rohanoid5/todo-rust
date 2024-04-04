use crate::{preclude::*, ui};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use tokio_postgres::Row;

pub struct App {
    pub selected_index: usize,
    pub tasks: Vec<Row>,
    pub exit: bool,
    pub db: Database,
}

impl App {
    pub fn new(tasks: Vec<Row>, db: Database) -> Self {
        Self {
            selected_index: 0,
            tasks,
            exit: false,
            db,
        }
    }

    /// runs the application's main loop until the user quits
    pub async fn run(&mut self, terminal: &mut ui::Tui) -> Result<(), Error> {
        while !self.exit {
            terminal.draw(|frame| self.render_frame(frame))?;
            self.handle_events().await?;
        }
        Ok(())
    }

    fn render_frame(&self, frame: &mut Frame) {
        // let layout = Layout::default()
        //     .direction(Direction::Horizontal)
        //     .constraints([Constraint::Length(20), Constraint::Min(0)])
        //     .split(frame.size());
        let mut state = ListState::default().with_selected(Some(self.selected_index));

        frame.render_stateful_widget(self, frame.size(), &mut state);
    }

    /// updates the application's state based on user input
    async fn handle_events(&mut self) -> Result<(), Error> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                // self.handle_key_event(key_event)
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Up => self.move_up(),
                    KeyCode::Down => self.move_down(),
                    KeyCode::Enter => {
                        let current_task = &self.tasks[self.selected_index];
                        let name = current_task.get::<&str, String>("name");
                        self.db.toggle_task(vec![name]).await?;

                        let tasks = self.db.get_all_tasks().await?;
                        self.tasks = tasks;
                    }
                    _ => {}
                }
            }
            _ => {}
        };
        Ok(())
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        } else {
            self.selected_index = self.tasks.len() - 1;
        }
    }

    fn move_down(&mut self) {
        if self.selected_index < (self.tasks.len() - 1) {
            self.selected_index += 1;
        } else {
            self.selected_index = 0;
        }
    }
}

impl StatefulWidget for &App {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
        let title = Title::from(" Terminal Todo List ".bold());
        let instructions = Title::from(Line::from(vec![
            " Mark task as Complete/Incomplete ".into(),
            "<Press Enter>".blue().bold(),
            " Delete task ".into(),
            "<Press Delete>".blue().bold(),
            " Quit ".into(),
            "<Press Q> ".blue().bold(),
        ]));
        let block = Block::default()
            .title(title.alignment(Alignment::Center))
            .title(
                instructions
                    .alignment(Alignment::Center)
                    .position(Position::Bottom),
            )
            .borders(Borders::ALL)
            .border_set(border::THICK);

        let lines: Vec<Line> = self
            .tasks
            .iter()
            .map(|task| {
                let current_task = Task::new(task.get(0), task.get(1), task.get(3));

                if current_task.checked {
                    Line::from(vec![current_task.name.crossed_out()])
                } else {
                    Line::from(vec![current_task.name.into()])
                }
            })
            .collect();

        ratatui::widgets::StatefulWidget::render(
            List::new(lines)
                .highlight_symbol("» ")
                .highlight_spacing(HighlightSpacing::Always)
                .block(block),
            area,
            buf,
            state,
        );
    }
}
