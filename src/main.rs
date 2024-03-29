use clap::{Args, Parser, Subcommand};
use preclude::Error;
// use dialoguer::Editor;
use std::future::IntoFuture;

mod db;
mod task;

mod preclude {
    pub use crate::db::*;
    pub use crate::task::*;
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
    ShowAll,
}

#[derive(Args)]
struct AddArgs {
    name: Option<Vec<String>>,
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

            for task in tasks {
                let current_task = Task::new(task.get(0), task.get(1), task.get(3));

                println!(
                    "{}, {}, {}",
                    current_task._id, current_task.name, current_task.checked
                );
            }
        }
        Some(Commands::ShowAll) => {
            let tasks = db.get_all_tasks().await?;
            for task in tasks {
                let current_task = Task::new(task.get(0), task.get(1), task.get(3));

                println!(
                    "{}, {}, {}",
                    current_task._id, current_task.name, current_task.checked
                );
            }
        }
        None => println!("NOTHING!!!"),
    }

    Ok(())
}
