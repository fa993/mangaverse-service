use std::collections::{HashMap, HashSet};

use mangaverse_entity::models::genre::Genre;
use sqlx::{QueryBuilder, MySql, Pool};
use uuid::Uuid;



pub async fn insert_genre(set: &HashSet<String>, pool: &Pool<MySql>) -> anyhow::Result<HashMap<String, Genre>> {
    
    let mut q = QueryBuilder::new("INSERT into genre(genre_id, name) ");

    q.push_values(set, |mut b, genre| {
        b.push_bind(Uuid::new_v4().to_string());
        b.push_bind(genre);
    });

    q.push(" ON DUPLICATE KEY update genre_id = genre_id");

    q.build().execute(pool).await?;

    let all = sqlx::query_as!(Genre, "SELECT genre.genre_id as id, genre.name from genre order by genre.name ASC").fetch_all(pool).await?;

    Ok(all.into_iter().map(|f| (f.name.to_string(), f)).collect::<HashMap<String, Genre>>())

}

