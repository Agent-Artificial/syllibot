use crate::types::{SubnetPost, SubnetPostData, Result};
use base64::{engine::general_purpose, Engine as _};
use log::info;

pub async fn text2text(text: &String, source_language: &String, target_language: &String) -> Result<String> {
	let subnet_post = SubnetPost {
		input: text.to_owned(),
		source_language: source_language.to_owned(),
		target_language: target_language.to_owned(),
		task_string: "text2text".to_string(),
	};
	let body = SubnetPostData {
		data: subnet_post,
	};

	info!("text2text::body::{:?}", body);

    let client = reqwest::Client::new();
    let response = client
        .post("https://miner-cellium.ngrok.app/modules/translation/process")
        .header("content-type", "application/json")
        .json(&body)
        .send().await?;

	info!("audio_to_text::post_response::{:?}", response);

    let response_text: String = response.text().await?;
    let value: serde_json::Value = serde_json::from_str(&response_text).unwrap();
    let json_str: String = value.as_str().unwrap().into();
	let decoded_bytes = general_purpose::STANDARD.decode(&json_str)?;
    let decoded_text = core::str::from_utf8(&decoded_bytes)?;
    info!("audio_to_text::decoded_text::{:?}", decoded_text);
    Ok(decoded_text.to_string())
}

