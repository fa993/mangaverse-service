use std::collections::HashMap;

use async_std::prelude::FutureExt;
use mangaverse_entity::models::genre::Genre;

pub mod manganelo;
pub mod readm;

#[async_std::main]
async fn main() -> anyhow::Result<()>{
    println!("Hello, world!");
    let f1 = manganelo::entity::get_manganelo_genre();
    let f2 = readm::entity::get_readm_genre();
    let mut r = f1.try_join(f2).await?;
    r.0.extend(r.1);
    let all = r.0.into_iter().map(|f| (f.clone(), Genre {
        id: String::new(),
        name: f
    })).collect::<HashMap<String, Genre>>();
    println!("{:#?}", all);
    Ok(())
}
