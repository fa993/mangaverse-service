use async_std::prelude::FutureExt;
use sqlx::mysql::MySqlPoolOptions;

use crate::db::genre::insert_genre;

pub mod manganelo;
pub mod readm;
pub mod db;

#[async_std::main]
async fn main() -> anyhow::Result<()>{
    println!("Hello, world!");

    let mut configs = dotenvy::dotenv_iter().expect("No env file found");

    let db_url = configs.find(|f| {
        if let Ok(t) = f {
            if t.0 == "DATABASE_URL" {
                return true;
            }
        }
        return false;
    }).expect("DATABASE_URL must be set").expect("DATABASE_URL must be set").1;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str()).await?;

    let f1 = manganelo::entity::get_manganelo_genre();
    let f2 = readm::entity::get_readm_genre();
    let mut r = f1.try_join(f2).await?;
    r.0.extend(r.1);
    let all = insert_genre(&r.0, &pool).await?;
    println!("{:#?}", all);
    Ok(())
}
