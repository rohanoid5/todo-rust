use crate::preclude::Error;
use tokio_postgres::{Client, NoTls, Row};

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("migrations");
}

pub struct Database {
    pub client: Client,
}

impl Database {
    pub async fn new(host: String, user: String) -> Result<Self, Error> {
        let (mut client, connection) =
            tokio_postgres::connect(&format!("host={} user={}", host, user), NoTls).await?;

        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        let migration_report = embedded::migrations::runner()
            .run_async(&mut client)
            .await?;

        for migration in migration_report.applied_migrations() {
            println!(
                "Migration Applied -  Name: {}, Version: {}",
                migration.name(),
                migration.version()
            );
        }

        Ok(Self { client })
    }

    pub async fn add_task(&self, name: Vec<String>) -> Result<(), Error> {
        self.client
            .execute("INSERT INTO todo (name) VALUES ($1)", &[&name.join(" ")])
            .await?;

        Ok(())
    }

    pub async fn get_all_tasks(&self) -> Result<Vec<Row>, Error> {
        let rows = self.client.query("SELECT * FROM todo", &[]).await?;

        Ok(rows)
    }

    pub async fn get_task_by_name(&self, name: Vec<String>) -> Result<Vec<Row>, Error> {
        let rows = self
            .client
            .query("SELECT * FROM todo where name = $1", &[&name.join(" ")])
            .await?;

        Ok(rows)
    }
}
