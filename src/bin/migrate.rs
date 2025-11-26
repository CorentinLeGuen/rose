use sea_orm::Database;
use sea_orm_migration::prelude::*;

use rose::migrator::Migrator;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = String::from("postgresql://root@localhost:26257/defaultdb?sslmode=disable");
    let db = Database::connect(&database_url).await?;

    Migrator::up(&db, None).await?;
    println!("Database migrated.");

    Ok(())
}
