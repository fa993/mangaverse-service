use std::collections::{HashMap, HashSet};

use isahc::prelude::*;
use mangaverse_entity::models::{
    chapter::ChapterTable, genre::Genre, manga::MangaTable, page::PageTable, source::SourceTable,
};
use scraper::{Html, Selector};
use sqlx::{
    types::chrono::{NaiveDateTime, Utc},
    MySql, Pool,
};

use crate::{readm::insert_source_if_not_exists, Error, Result};

const AUTHOR: &str = "Author(s) :";
const ALTERNATIVE_NAME: &str = "Alternative :";
const STATUS: &str = "Status :";
const GENRES: &str = "Genres :";
const UPDATED: &str = "Updated :";
const SOURCE_NAME: &str = "manganelo";

pub async fn get_manganelo_genre() -> Result<HashSet<String>> {
    let url = "https://manganato.com/genre-all";

    let response_text = isahc::get_async(url).await?.text().await?;

    let doc = Html::parse_document(&response_text);

    let genre_selector = Selector::parse("div.advanced-search-tool-genres-list > span").unwrap();

    Ok(doc
        .select(&genre_selector)
        .map(|f| f.text().collect::<String>().trim().to_lowercase())
        .collect())
}

pub async fn get_manganelo_source(pool: &Pool<MySql>) -> Result<SourceTable> {
    insert_source_if_not_exists(SOURCE_NAME, 2, pool).await
}

pub async fn get_manga<'a>(
    url: String,
    sc: &'a SourceTable,
    map: &'a HashMap<String, Genre>,
) -> Result<MangaTable<'a>> {
    let mut mng: MangaTable = MangaTable::new(sc);
    mng.is_listed = true;
    mng.url = url;

    let doc = Html::parse_document(
        isahc::get_async(mng.url.as_str())
            .await?
            .text()
            .await?
            .as_str(),
    );

    let name_selector = Selector::parse("div.story-info-right > h1").unwrap();

    mng.name.extend(
        doc.select(&name_selector)
            .next()
            .ok_or(Error::TextParseError)?
            .text(),
    );

    mng.name = mng.name.trim().to_string();

    mng.titles.push(mng.name.clone());

    mng.cover_url.push_str(
        doc.select(&Selector::parse("span.info-image > img").unwrap())
            .next()
            .and_then(|f| f.value().attr("src"))
            .ok_or(Error::TextParseError)?,
    );

    let table_label_selector = Selector::parse("td.table-label").unwrap();
    let table_value_selector = Selector::parse("td.table-value").unwrap();

    let iter_label = doc.select(&table_label_selector);
    let iter_value = doc.select(&table_value_selector);

    let metadata_table = iter_label.zip(iter_value);

    for (label, value) in metadata_table {
        match label.text().collect::<String>().as_str() {
            AUTHOR => mng.authors.extend(
                value
                    .text()
                    .collect::<String>()
                    .split(&['-'])
                    .map(str::trim)
                    .map(ToString::to_string),
            ),
            ALTERNATIVE_NAME => mng.titles.extend(
                value
                    .text()
                    .collect::<String>()
                    .split(&[',', ';'])
                    .map(str::trim)
                    .map(ToString::to_string),
            ),
            STATUS => mng
                .status
                .extend(value.text().map(|f| f.trim().to_uppercase())),
            GENRES => mng.genres.extend(
                value
                    .text()
                    .collect::<String>()
                    .split(&['-'])
                    .map(str::trim)
                    .map(str::to_lowercase)
                    .filter_map(|f| map.get(&f)),
            ),
            _ => {}
        };
    }

    let table_label_selector = Selector::parse("span.stre-label").unwrap();
    let table_value_selector = Selector::parse("span.stre-value").unwrap();

    let iter_label = doc.select(&table_label_selector);
    let iter_value = doc.select(&table_value_selector);

    let metadata_table = iter_label.zip(iter_value);

    for (label, value) in metadata_table {
        if label.text().collect::<String>() == UPDATED {
            let y = value.text().collect::<String>();
            let x = y[0..y.len() - 3].trim();
            mng.last_updated = NaiveDateTime::parse_from_str(x, "%b %d,%Y - %H:%M").ok();
        }
    }

    let desc_sel = Selector::parse(".panel-story-info-description").unwrap();

    if let Some(x) = doc.select(&desc_sel).next() {
        let mut u = x.text().collect::<String>();
        u.drain(0..=13);
        mng.description.push_str(u.as_str().trim());
    }

    let table_label_selector = Selector::parse("a.chapter-name").unwrap();
    let table_value_selector = Selector::parse("span.chapter-time").unwrap();

    let iter_label = doc.select(&table_label_selector);
    let iter_value = doc.select(&table_value_selector);

    let images_selector = Selector::parse("div.container-chapter-reader > img").unwrap();

    let chapter_table = iter_label.zip(iter_value);

    for (idx, (t1, t2)) in chapter_table.enumerate() {
        let mut t = ChapterTable {
            sequence_number: idx as i32,
            last_watch_time: Utc::now().timestamp_millis(),
            updated_at: NaiveDateTime::parse_from_str(
                t2.value().attr("title").unwrap_or("").trim(),
                "%b %d,%Y %H:%M",
            )
            .ok(),
            ..Default::default()
        };

        let t1_text = t1.text().collect::<String>();
        let y = t1_text.find(':');
        let chp = t1_text.find("Chapter");
        t.chapter_name = t1_text.clone();
        if chp.is_none() {
            t.chapter_name = t1_text;
        } else if y.is_some() {
            let act_y = y.unwrap();
            let act_chp = chp.unwrap();
            if act_y + 2 < t1_text.len() {
                t.chapter_name = t1_text[act_y + 2..].trim().to_string();
            }
            if act_y < t1_text.len() && act_chp + "Chapter".len() + 1 < t1_text.len() {
                t.chapter_number = t1_text[act_chp + "Chapter".len() + 1..act_y]
                    .trim()
                    .to_string();
            }
        } else {
            let act_chp = chp.unwrap();
            if act_chp + "Chapter".len() + 1 < t1_text.len() {
                t.chapter_number = t1_text[act_chp + "Chapter".len() + 1..].trim().to_string()
            }
        }

        if let Some(url_chp) = t1.value().attr("href") {
            t.pages = Html::parse_document(isahc::get_async(url_chp).await?.text().await?.as_str())
                .select(&images_selector)
                .filter_map(|f| f.value().attr("src"))
                .map(ToString::to_string)
                .enumerate()
                .map(|(idx, u)| PageTable {
                    url: u,
                    page_number: idx as i32,
                    ..Default::default()
                })
                .collect();
        }

        mng.chapters.push(t);
    }

    mng.chapters.reverse();

    let sz = mng.chapters.len() as i32;

    for t in mng.chapters.iter_mut() {
        t.sequence_number = sz - t.sequence_number - 1;
    }

    Ok(mng)
}
