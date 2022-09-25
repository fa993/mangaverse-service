use std::collections::HashSet;

use isahc::prelude::*;
use scraper::{Html, Selector};

pub async fn get_manganelo_genre() -> anyhow::Result<HashSet<String>> {

    let url = "https://manganato.com/genre-all";

    let response_text = isahc::get_async(url).await?.text().await?;

    let doc = Html::parse_document(&response_text);

    let genre_selector = Selector::parse("div.advanced-search-tool-genres-list > span").unwrap();

    Ok(doc.select(&genre_selector).map(|f| f.text().collect::<String>().trim().to_lowercase()).map(Into::into).collect())

}