use app::App;
use clap::{Args, Parser, Subcommand};
use preclude::Error;
// use dialoguer::Editor;
use console::style;
use std::future::IntoFuture;
use std::io::Result as IOResult;
use tokio_postgres::Row;

mod app;
mod db;
mod task;
mod ui;

mod preclude {
    pub use crate::app::*;
    pub use crate::db::*;
    pub use crate::task::*;
    pub use crate::ui::*;
    pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
}

use db::*;
use task::*;

const DB_HOST: &str = "localhost";
const DB_USER: &str = "postgres";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Add(AddArgs),
    Search(AddArgs),
    Done(AddArgs),
    ShowAll,
}

#[derive(Args)]
struct AddArgs {
    name: Option<Vec<String>>,
}

fn print_rows(tasks: Vec<Row>) {
    for task in tasks {
        let current_task = Task::new(task.get(0), task.get(1), task.get(3));

        if current_task.checked == true {
            println!(
                "{}. {}",
                current_task._id,
                style(current_task.name).green().strikethrough()
            );
        } else {
            println!("{}. {}", current_task._id, style(current_task.name).blue());
        }
    }
}

async fn ui(tasks: Vec<Row>, db: Database) -> Result<(), Error> {
    let mut terminal = ui::init()?;
    let _ = App::new(tasks, db).run(&mut terminal).await?;
    ui::restore()?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    // if let Some(rv) = Editor::new().edit("Enter a commit message").unwrap() {
    //     println!("Your message:");
    //     println!("{}", rv);
    // } else {
    //     println!("Abort!");
    // }

    let client = Database::new(DB_HOST.to_owned(), DB_USER.to_owned());
    let db = client.into_future().await?;

    match &cli.command {
        Some(Commands::Add(arg)) => {
            let task_name = arg.name.clone().unwrap_or(Vec::new());
            db.add_task(task_name).await?;
        }
        Some(Commands::Search(arg)) => {
            let task_name = arg.name.clone().unwrap_or(Vec::new());
            let tasks = db.get_task_by_name(task_name).await?;

            print_rows(tasks);
        }
        Some(Commands::Done(arg)) => {
            let task_name = arg.name.clone().unwrap_or(Vec::new());

            db.toggle_task(true, task_name).await?;
        }
        Some(Commands::ShowAll) => {
            let tasks = db.get_all_tasks().await?;
            print_rows(tasks);
        }
        None => {
            let tasks = db.get_all_tasks().await?;
            let _ = ui(tasks, db).await?;
        }
    }

    Ok(())
}
