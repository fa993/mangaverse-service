use mangaverse_entity::models::chapter::ChapterTable;
use sqlx::{MySql, Pool, QueryBuilder};

use crate::Result;

pub async fn update(ori: ChapterTable, lat: ChapterTable, pool: &Pool<MySql>) -> Result<()> {
    let chk_met = ori.chapter_name == lat.chapter_name
        && ori.chapter_number == lat.chapter_number
        && ori.updated_at == lat.updated_at;

    let chk_pg = ori
        .pages
        .iter()
        .zip(lat.pages.iter())
        .all(|e| e.0.url == e.1.url);

    if !chk_met {
        sqlx::query!("UPDATE chapter SET chapter_name = ?, chapter_number = ?, updated_at = ? where chapter_id = ?", lat.chapter_name, lat.chapter_number, lat.updated_at, ori.chapter_id).execute(pool).await?;
    }

    if !chk_pg {
        //remove previous...
        sqlx::query!(
            "DELETE FROM chapter_page where chapter_id = ?",
            ori.chapter_id
        )
        .execute(pool)
        .await?;

        //add new
        let mut q = QueryBuilder::new("INSERT into chapter_page(url, page_number, chapter_id) ");

        q.push_values(lat.pages, |mut b, page| {
            b.push_bind(page.url);
            b.push_bind(page.page_number);
            b.push_bind(page.chapter_id);
        });

        q.build().execute(pool).await?;
    }

    Ok(())
}
