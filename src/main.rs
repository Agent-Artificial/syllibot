use poise::serenity_prelude as serenity;
use dotenv::dotenv;
use core::str;
use std::io::Cursor;
use log::info;
use base64::{engine::general_purpose, Engine as _};
use tokio::io::AsyncReadExt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct AudioPost {
	input: String,
	source_language: String,
	target_language: String,
	task_string: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AudioPostData {
	data: AudioPost,
}

struct Data {} // User data, which is stored and accessible in all command invocations
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

async fn fetch_file(url: String, filename: &String) -> Result<()> {
    let response = reqwest::get(&url).await?;
    let mut file = std::fs::File::create(&filename)?;
    let mut content = Cursor::new(response.bytes().await?);
    std::io::copy(&mut content, &mut file)?;
    info!("fetch_file::{} saved", filename);
    Ok(())
}

async fn delete_file(filename: &String) -> Result<()> {
    std::fs::remove_file(&filename)?;
    info!("delete_file::{} deleted", filename);
    Ok(())
}

async fn autocomplete_language<'a>(
    _ctx: Context<'_>,
    partial: &str,
) -> Vec<String> {
	let available_languages = vec![
		"English",
		"Cantonese",
		"French",
		"German",
		"Hindi",
		"Italian",
		"Japanese",
		"Korean",
		"Mandarin Chinese",
		"Russian",
		"Spanish",
		"Afrikaans",
		"Amharic",
		"Armenian",
		"Assamese",
		"Asturian",
		"Basque",
		"Belarusian",
		"Bengali",
		"Bosnian",
		"Bulgarian",
		"Burmese",
		"Catalan",
		"Cebuano",
		"Central",
		"Colloquial Malay",
		"Croatian",
		"Czech",
		"Danish",
		"Dutch",
		"Egyptian Arabic",
		"Estonian",
		"Finnish",
		"Galician",
		"Ganda",
		"Georgian",
		"Gujarati",
		"Halh Mongolian",
		"Hebrew",
		"Hungarian",
		"Icelandic",
		"Igbo",
		"Indonesian",
		"Irish",
		"Javanese",
		"Kabuverdianu",
		"Kamba",
		"Kannada",
		"Kazakh",
		"Khmer",
		"Kyrgyz",
		"Lao",
		"Lithuanian",
		"Luo",
		"Luxembourgish",
		"Macedonian",
		"Maithili",
		"Malayalam",
		"Maltese",
		"Mandarin Chinese Hant",
		"Marathi",
		"Meitei",
		"Modern Standard Arabic",
		"Moroccan Arabic",
		"Nepali",
		"Nigerian Fulfulde",
		"North Azerbaijani",
		"Northern Uzbek",
		"Norwegian Bokmål",
		"Norwegian Nynorsk",
		"Nyanja",
		"Occitan",
		"Odia",
		"Polish",
		"Portuguese",
		"Punjabi",
		"Romanian",
		"Serbian",
		"Shona",
		"Sindhi",
		"Slovak",
		"Slovenian",
		"Somali",
		"Southern Pashto",
		"Standard Latvian",
		"Standard Malay",
		"Swahili",
		"Swedish",
		"Tagalog",
		"Tajik",
		"Tamil",
		"Telugu",
		"Thai",
		"Turkish",
		"Ukrainian",
		"Urdu",
		"Vietnamese",
		"Welsh",
		"West Central Oromo",
		"Western Persian",
		"Xhosa",
		"Yoruba",
		"Zulu",
    ];

    available_languages.into_iter()
		.filter(move |name| name.starts_with(partial))
		.map(String::from)
		.collect()
}

/// Transcribes an audio file
#[poise::command(slash_command, prefix_command)]
async fn audio_to_text(
    ctx: Context<'_>,
    #[description = "Audio File"]
	file: serenity::Attachment,
	#[description = "Source Language"]
	#[autocomplete = "autocomplete_language"]
	source_language: String,
    #[description = "Target Language"]
	#[autocomplete = "autocomplete_language"]
	target_language: String,

) -> Result<()> {
    let filename = file.filename;
    let url = file.url;
    let filesize = file.size;
    info!("audio_to_text::filename({})::filesize({})", &filename, filesize);
    fetch_file(url, &filename).await?;

    let mut file = tokio::fs::File::open(&filename).await?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;
    let encoded_data = general_purpose::STANDARD.encode(&bytes);

	let audio_post = AudioPost {
		input: encoded_data,
		source_language,
		target_language,
		task_string: "speech2text".to_string(),
	};
	let body = AudioPostData {
		data: audio_post,
	};

	info!("audio_to_text::body::{:?}", body);

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
    info!("audio_to_text::response_text::{:?}", &json_str);
	let decoded_bytes = general_purpose::STANDARD.decode(&json_str)?;
    let decoded_text = str::from_utf8(&decoded_bytes)?;
    info!("audio_to_text::decoded_text::{:?}", decoded_text);

    delete_file(&filename).await?;

    ctx.say(decoded_text).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "syllibot=info")
    }
    env_logger::init();
    info!("Initializing...");
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![audio_to_text()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    info!("Client initialized, starting bot.");
    client.unwrap().start().await.unwrap();
}
