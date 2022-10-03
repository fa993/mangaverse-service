use std::collections::HashMap;

use crate::db::genre::insert_genre;
use async_std::prelude::FutureExt;
use mangaverse_entity::models::{source::SourceTable, genre::Genre};
use sqlx::mysql::MySqlPoolOptions;
use thiserror::Error;
use tuple_conv::RepeatedTuple;

pub mod db;
pub mod manganelo;
pub mod readm;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {

    #[error("Text Parse Error")]
    TextParseError,

    #[error(transparent)]
    SQLError(#[from] sqlx::Error),

    #[error(transparent)]
    NetworkError(#[from] isahc::Error),

    #[error(transparent)]
    IOError(#[from] std::io::Error),

    #[error(transparent)]
    OtherError(#[from] Box<dyn std::error::Error>),

    #[error("You shouldn't be seeing this")]
    NoError

}

#[derive(Default, Debug)]
pub struct Context {

    sources: HashMap<String, SourceTable>,
    genres: HashMap<String, Genre>,
    

}

async fn setup_db() -> Result<sqlx::Pool<sqlx::MySql>> {
    let configs = dotenvy::dotenv_iter().expect("No env file found");

    let db_url = configs
        .filter_map(std::result::Result::ok)
        .find(|f| f.0 == "DATABASE_URL")
        .expect("DATABASE_URL must be set")
        .1;

    let pool = MySqlPoolOptions::new()
        .max_connections(5)
        .connect(db_url.as_str())
        .await?;
    Ok(pool)
}

#[async_std::main]
async fn main() -> Result<()> {
    println!("Hello, world!");

    let pool = setup_db().await?;

    let mut c = Context::default();

    let f1 = manganelo::entity::get_manganelo_genre();
    let f2 = readm::entity::get_readm_genre();
    let r = f1.try_join(f2).await?.to_vec().into_iter().flatten().collect();

    insert_genre(&r, &pool, &mut c.genres).await?;

    let g1 = manganelo::entity::get_manganelo_source(&pool);
    let g2 = readm::entity::get_readm_source(&pool);

    let s = g1.try_join(g2).await?;
    
    c.sources = s.to_vec().into_iter().map(|f| (f.name.clone(), f)).collect();

    println!("{:#?}", c);
    Ok(())
}
