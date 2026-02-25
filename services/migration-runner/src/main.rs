use sqlx::postgres::PgPoolOptions;
use sqlx::migrate::Migrator;
use std::path::Path;
use dotenvy::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    // 1. Main DB Migrations
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    println!("Running migrations for Main DB...");
    let pool = PgPoolOptions::new().connect(&db_url).await?;
    
    let migrator = Migrator::new(Path::new("./migrations")).await?;
    migrator.run(&pool).await?;
    println!("Main DB migrations applied!");

    Ok(())
}
