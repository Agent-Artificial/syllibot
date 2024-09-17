use std::io::Cursor;
use log::info;
use crate::types::Result;

pub async fn fetch_file(url: String, filename: &String) -> Result<()> {
    let response = reqwest::get(&url).await?;
    let mut file = std::fs::File::create(&filename)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    info!("fetch_file::{} saved", filename);
    Ok(())
}

pub async fn delete_file(filename: &String) -> Result<()> {
    std::fs::remove_file(&filename)?;
    info!("delete_file::{} deleted", filename);
    Ok(())
}

