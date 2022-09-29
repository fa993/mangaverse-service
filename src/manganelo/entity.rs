use std::collections::{HashSet, HashMap};

use isahc::prelude::*;
use mangaverse_entity::models::{manga::MangaTable, source::SourceTable, genre::Genre};
use scraper::{Html, Selector};

use crate::{Result, Error};

const AUTHOR: &str = "Author(s) :";
const ALTERNATIVE_NAME: &str = "Alternative :";
const STATUS: &str = "Status :";
const GENRES: &str = "Genres :";
const UPDATED: &str = "Updated :";

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

pub async fn get_manganelo_source() -> Result<SourceTable> {
	todo!()
}

/*

        boolean error = false;
		Exception e = null;
		MangaDTO mndt = new MangaDTO();
		mndt.setURL(url);
		mndt.setDescription("");
		mndt.setSource(s);
		try {
			Document el = Jsoup.connect(url).get();
			Optional.ofNullable(el.selectFirst("div.story-info-right > h1")).map(t -> t.text().strip())
					.ifPresentOrElse(t -> {
						mndt.setPrimaryTitle(t);
						mndt.getTitles().add(t);
					}, () -> {
						throw new RuntimeException("No title");
					});
			Optional.ofNullable(el.selectFirst("span.info-image > img"))
					.ifPresentOrElse(t -> mndt.setCoverURL(t.attr("src")), () -> {
						throw new RuntimeException("No Cover URL");
					});
			Elements labels = el.select("td.table-label");
			Elements values = el.select("td.table-value");
			for (int i = 0; i < labels.size(); i++) {
				switch (labels.get(i).text()) {
				case AUTHOR -> extractWithTrim(values.get(i).text(), mndt.getAuthors(), EMPTY_SET, '-');
				case ALTERNATIVE_NAME -> extractWithTrim(values.get(i).text(), mndt.getTitles(), EMPTY_SET, ',', ';');
				case STATUS -> mndt.setStatus(values.get(i).text().toUpperCase());
				case GENRES -> extractWithTrim(values.get(i).text(), mndt.getGenres(), EMPTY_SET, '-');
				default -> System.out.println("Not recognized");
				}
			}
			Elements labels2 = el.select("span.stre-label");
			Elements values2 = el.select("span.stre-value");
			for (int i = 0; i < labels2.size(); i++) {
				switch (labels2.get(i).text()) {
				case UPDATED:
					String x = values2.get(i).text();
					mndt.setLastUpdated(FMT.parse(x.substring(0, x.length() - 3), Instant::from));
					break;
				default:
				}
			}
			Optional.ofNullable(el.selectFirst(".panel-story-info-description")).ifPresent(t -> {
				mndt.setDescription(t.text().substring(13).strip());
			});
			Elements ls = el.select("a.chapter-name");
			Elements ls2 = el.select("span.chapter-time");
			for (int i = 0; i < ls.size(); i++) {
				Element t1 = ls.get(i);
				Element t2 = ls2.get(i);
				String x = t1.text();
				String chapName = "";
				String chapNumber = "";
				try {
					int y = x.indexOf(':');
					int chp = x.indexOf("Chapter");
					if (chp < 0) {
						chapName = x;
					} else if (y > -1) {
						chapName = x.substring(y + 2).strip();
						chapNumber = x.substring(chp + "Chapter".length() + 1, y).strip();
					} else {
						chapNumber = x.substring(chp + "Chapter".length() + 1).strip();
					}
				} catch (Exception ex) {
					ex.printStackTrace();
					System.out.println("Happened here: " + x + " with manga " + mndt);
					chapName = x;
					e = ex;
					error = true;
				}
				ChapterDTO cdt = new ChapterDTO();
				cdt.setChapterName(chapName);
				cdt.setChapterNumber(chapNumber);
				cdt.setSequenceNumber(ls.size() - i - 1);
				Optional.ofNullable(t2.attr("title")).ifPresent(t -> cdt.setUpdatedAt(FMTC.parse(t, Instant::from)));
				try {
					cdt.setImagesURL(getImages(t1.attr("abs:href")));
				} catch (Exception ex) {
					ex.printStackTrace();
					error = true;
					e = ex;
				}
				mndt.getChapters().add(cdt);
			}
		} catch (Exception ex) {
			System.out.println("Happened Here: " + mndt);
			throw new MangaFetchingException(url, mndt, ex);
		}
		if (error) {
			throw new MangaFetchingException(url, mndt, e);
		}
		return mndt;

*/

pub async fn get_manga<'a>(url: String, sc: &'a SourceTable, map: &'a HashMap<String, Genre>) -> Result<MangaTable<'a>> {

	let mut mng: MangaTable = MangaTable::new(sc);

	// let mut man = MangaTable::default();

    let doc = Html::parse_document(isahc::get_async(url.as_str()).await?.text().await?.as_str());

	let name_selector = Selector::parse("div.story-info-right > h1").unwrap();

	mng.name.extend(doc.select(&name_selector).next().ok_or(Error::TextParseError)?.text());

	mng.titles.push(mng.name.clone());

	mng.cover_url.push_str(doc.select(&Selector::parse("span.info-image > img").unwrap()).next().and_then(|f| f.value().attr("src")).ok_or(Error::TextParseError)?);

	let table_label_selector = Selector::parse("td.table-label").unwrap();
	let table_value_selector = Selector::parse("td.table-value").unwrap();

	let iter_label = doc.select(&table_label_selector);
	let iter_value = doc.select(&table_value_selector);

	let metadata_table = iter_label.zip(iter_value);



	// for (label, value) in metadata_table {
	// 	match label.text().collect::<String>() {
	// 		AUTHOR => ,
	// 		STATUS => 
	// 	}
	// }
	
	Ok(mng)
}