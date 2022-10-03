use std::collections::HashSet;

use isahc::prelude::*;
use mangaverse_entity::models::source::SourceTable;
use scraper::{Html, Selector};
use sqlx::{Pool, MySql};

use crate::Result;

use super::insert_source_if_not_exists;

const SOURCE_NAME: &str = "readm";

pub async fn get_readm_source(pool: &Pool<MySql>) -> Result<SourceTable> {
	insert_source_if_not_exists(SOURCE_NAME, 1, pool).await
}

pub async fn get_readm_genre() -> Result<HashSet<String>> {
    let url = "https://readm.org/advanced-search";

    let response_text = isahc::get_async(url).await?.text().await?;

    let doc = Html::parse_document(&response_text);

    let genre_selector = Selector::parse("ul.advanced-search-categories li").unwrap();

    Ok(doc
        .select(&genre_selector)
        .filter_map(|f| {
            let r = f.text().collect::<String>().trim().to_lowercase();
            if r == "uncategorized" {
                None
            } else {
                Some(r)
            }
        })
        .collect())
}
