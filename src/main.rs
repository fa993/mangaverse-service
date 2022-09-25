use futures::try_join;

pub mod manganelo;
pub mod readm;

#[async_std::main]
async fn main() -> anyhow::Result<()>{
    println!("Hello, world!");
    let f1 = manganelo::entity::get_manganelo_genre();
    let f2 = readm::entity::get_readm_genre();
    let mut r = try_join!(f1, f2)?;
    r.0.extend(r.1);
    println!("{:?}", r.0);
    Ok(())
}
