use poise::serenity_prelude::{self as serenity, ComponentInteractionDataKind};
use dotenv::dotenv;
use core::str;
use std::io::Cursor;
use log::info;
use base64::{engine::general_purpose, Engine as _};
use tokio::io::AsyncReadExt;
use serde::{Deserialize, Serialize};

mod language_detection;

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

pub fn get_available_languages() -> Vec<&'static str> {
    let available_languages = vec![
		"English",
		"Polish",
		"French",
        "German",
		"Spanish",
		"Romanian",
		"Turkish",
		"Dutch",
		"Swedish",
		"Slovenian",
    ];
    available_languages
}

async fn autocomplete_language<'a>(
    _ctx: Context<'_>,
    partial: &str,
) -> Vec<String> {
	let available_languages = get_available_languages();

    available_languages.into_iter()
		.filter(move |name| name.starts_with(partial))
		.map(String::from)
		.collect()
}

#[derive(Debug, poise::Modal)]
struct TranslationModal {
    source_language: String,
    target_language: String,
}

pub fn language_select_menu_options() -> Vec<serenity::CreateSelectMenuOption> {
    let available_languages = get_available_languages();
    available_languages.into_iter().map(|lang| serenity::CreateSelectMenuOption::new(lang, lang)).take(25).collect()
}

pub async fn text2text(text: &String, source_language: &String, target_language: &String) -> Result<String> {
	let audio_post = AudioPost {
		input: text.to_owned(),
		source_language: source_language.to_owned(),
		target_language: target_language.to_owned(),
		task_string: "text2text".to_string(),
	};
	let body = AudioPostData {
		data: audio_post,
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
    let decoded_text = str::from_utf8(&decoded_bytes)?;
    info!("audio_to_text::decoded_text::{:?}", decoded_text);
    Ok(decoded_text.to_string())
}

/// List supported languages
#[poise::command(slash_command, prefix_command)]
pub async fn supported_languages(
    ctx: Context<'_>,
) -> Result<()> {
    let available_languages = get_available_languages();
    let reply = poise::CreateReply::default()
            .content(available_languages.iter().map(|i| format!("- {}", i)).collect::<Vec<String>>().join("\n"))
            .ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

/// Translate a message (hint: Right click a message and go to Apps -> Translate)
#[poise::command(context_menu_command = "Translate", slash_command, prefix_command)]
pub async fn translate_message(
    ctx: Context<'_>,
    #[description = "Message to translate (enter a link or ID)"]
    msg: serenity::Message,
) -> Result<()> {
    let interaction_id = ctx.id();
    let message_content = &msg.content;

    match language_detection::detect_language(message_content) {
        Ok(detected_language) => {
            log::info!("translate_message::detected_language::{:?}", detected_language);

            let reply = {
                let language_select_menu_options = language_select_menu_options();
                let components = vec![serenity::CreateActionRow::SelectMenu(
                    serenity::CreateSelectMenu::new(
                        format!("target_language_selector_{}", interaction_id),
                        serenity::CreateSelectMenuKind::String {
                            options: language_select_menu_options
                        }
                    )
                )];

                poise::CreateReply::default()
                    .content("Select the language to translate to with the dropdown below.")
                    .components(components)
                    .ephemeral(true)
            };

            let ephemeral_reply = ctx.send(reply).await?;

            while let Some(component_interaction) = serenity::ComponentInteractionCollector::new(ctx.serenity_context())
                .author_id(ctx.author().id)
                .channel_id(ctx.channel_id())
                .timeout(std::time::Duration::from_secs(120))
                .filter(move |component_interaction| component_interaction.data.custom_id == format!("target_language_selector_{}", interaction_id))
                .await
                {
                    let cidk = &component_interaction.data.kind;
                    let chosen_value =  match cidk {
                        ComponentInteractionDataKind::StringSelect { values } => values[0].clone(),
                        _ => "English".to_string(),
                    };
                    info!("translate_message::component_interaction::chosen_value::{}", chosen_value);
                    ephemeral_reply.edit(
                        ctx,
                        poise::CreateReply::default()
                            .content(format!("Translating from {} to {}...", &detected_language, &chosen_value))
                            .ephemeral(true)
                            .components(vec![])
                        ).await?;
                    component_interaction.defer_ephemeral(ctx).await?;
                    let translated_text = text2text(message_content, &detected_language, &chosen_value).await?;
                    msg.reply(ctx, format!("{}", translated_text)).await?;
                    info!("translate_message::sent_reply");
                    ephemeral_reply.delete(ctx).await?;
                    component_interaction.delete_response(ctx).await?;
                }

            Ok(())
        },
        Err(err) => {
            msg.reply(ctx, err).await?;
            Ok(())
        }
    }
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
            commands: vec![
                audio_to_text(),
                translate_message(),
                supported_languages(),
            ],
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
