use std::collections::HashSet;

use isahc::prelude::*;
use scraper::{Html, Selector};

pub async fn get_readm_genre() -> anyhow::Result<HashSet<String>> {

    let url = "https://readm.org/advanced-search";

    let response_text = isahc::get_async(url).await?.text().await?;

    let doc = Html::parse_document(&response_text);

    let genre_selector = Selector::parse("ul.advanced-search-categories li").unwrap();

    Ok(doc.select(&genre_selector).filter_map(|f| {
        let r = f.text().collect::<String>().trim().to_lowercase();
        if r == "uncategorized" {
            None
        } else {
            Some(r)
        }
    }).map(Into::into).collect())
}