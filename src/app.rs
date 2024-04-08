use crate::{preclude::*, ui};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::*,
    symbols::border,
    widgets::{block::*, *},
};
use tokio_postgres::Row;

pub enum AppState {
    Add,
    Edit,
    View,
}

pub struct App<'a> {
    pub selected_index: usize,
    pub tasks: Vec<Row>,
    pub exit: bool,
    pub db: Database,
    pub state: AppState,
    pub block: Block<'a>,
    pub task_name: String,
}

impl<'a> App<'a> {
    pub fn new(tasks: Vec<Row>, db: Database) -> Self {
        let title = Title::from(" Terminal Todo List ".bold());
        let instructions = Title::from(Line::from(vec![
            " View Tasks ".into(),
            "<Esc>".blue().bold(),
            " Add Task ".into(),
            "<A>".blue().bold(),
            " Edit Task ".into(),
            "<E>".blue().bold(),
            " Delete Task ".into(),
            "<D>".blue().bold(),
            " Complete/Incomplete Task ".into(),
            "<Enter>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
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

        Self {
            selected_index: 0,
            tasks,
            exit: false,
            db,
            state: AppState::View,
            block,
            task_name: String::new(),
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
        //     .constraints([Constraint::Length(100)])
        //     .split(frame.size());

        match self.state {
            AppState::View => {
                let mut state = ListState::default().with_selected(Some(self.selected_index));
                frame.render_stateful_widget(self, frame.size(), &mut state);
            }
            AppState::Add => {
                let title = Paragraph::new(Text::styled(
                    format!("Create New Task\n {}", self.task_name),
                    Style::default().fg(Color::Blue),
                ))
                .block(self.block.clone());
                frame.render_widget(title, frame.size());
            }
            AppState::Edit => {}
        }
    }

    async fn handle_events(&mut self) -> Result<(), Error> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => match self.state {
                AppState::View => self.handle_view_state(key_event).await?,
                AppState::Add => self.handle_add_state(key_event).await?,
                AppState::Edit => self.handle_view_state(key_event).await?,
            },
            _ => {}
        };
        Ok(())
    }

    async fn handle_view_state(&mut self, key_event: KeyEvent) -> Result<(), Error> {
        match key_event.code {
            KeyCode::Char('a') => {
                self.state = AppState::Add;
            }
            KeyCode::Char('e') => {
                self.state = AppState::Edit;
            }
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
        Ok(())
    }

    async fn handle_add_state(&mut self, key_event: KeyEvent) -> Result<(), Error> {
        match key_event.code {
            KeyCode::Char(value) => self.task_name.push(value),
            KeyCode::Backspace => {
                if self.task_name.len() > 0 {
                    self.task_name.pop();
                }
            }
            KeyCode::Enter => {
                self.db.add_task(vec![self.task_name.to_string()]).await?;

                let tasks = self.db.get_all_tasks().await?;
                self.tasks = tasks;
                self.state = AppState::View;
            }
            KeyCode::Esc => {
                self.state = AppState::View;
            }
            _ => {}
        }
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

impl<'a> StatefulWidget for &App<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ListState) {
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
                .block(self.block.clone()),
            area,
            buf,
            state,
        );
    }
}
