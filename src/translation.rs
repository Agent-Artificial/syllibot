use crate::types::{SubnetPost, Result};
use log::info;

pub async fn text2text(mainnet_api_url: String, text: &String, source_language: &String, target_language: &String) -> Result<String> {
	let body = SubnetPost {
		input: text.to_owned(),
		source_language: source_language.to_owned(),
		target_language: target_language.to_owned(),
		task_string: "text2text".to_string(),
	};

	info!("text2text::body::{:?}", body);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/translation", mainnet_api_url))
        .header("content-type", "application/json")
        .json(&body)
        .send().await?;

	info!("text2text::post_response::{:?}", response);

    let response_text: String = response.text().await?;
    let value: serde_json::Value = serde_json::from_str(&response_text).unwrap();
    let json_str: String = value.as_str().unwrap().into();
    info!("audio_to_text::decoded_text::{:?}", json_str);
    Ok(json_str.to_string())
}

