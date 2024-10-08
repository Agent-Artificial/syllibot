// Copyright: (c) 2024, J. Zane Cook <z@agentartificial.com>
// GNU General Public License v3.0+ (see COPYING or https://www.gnu.org/licenses/gpl-3.0.txt)
use ::config::Config;
use config::BotConfig;
use language_detection::detect_language;
use poise::serenity_prelude::{self as serenity, ComponentInteractionDataKind};
use dotenv::dotenv;
use log::info;
use tokio::io::AsyncReadExt;
use tokio_postgres::NoTls;
use base64::{engine::general_purpose, Engine as _};

mod language_detection;
mod types;
mod translation;
mod files;
mod config;

use types::{
    SubnetPost,
    Data,
    Result,
    Error,
    Context,
};
use translation::text2text;
use files::{fetch_file, delete_file};


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
        "Portuguese",
    ];
    available_languages
}

pub fn get_available_flags() -> Vec<&'static str> {
    let available_flags = vec![
        "🇺🇸",
        "🇵🇱",
        "🇫🇷",
        "🇩🇪",
        "🇲🇽",
        "🇷🇴",
        "🇹🇷",
        "🇳🇱",
        "🇸🇪",
        "🇸🇮",
        "🇵🇹",
    ];
    available_flags
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

pub fn language_select_menu_options() -> Vec<serenity::CreateSelectMenuOption> {
    let available_languages = get_available_languages();
    available_languages.into_iter().map(|lang| serenity::CreateSelectMenuOption::new(lang, lang)).take(25).collect()
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

/// Translate text into a language of your choice.
#[poise::command(slash_command, prefix_command)]
pub async fn translate_text(
    ctx: Context<'_>,
    #[description = "Text to translate"]
    text: String,
    #[description = "Target Language"]
	#[autocomplete = "autocomplete_language"]
	target_language: String,
) -> Result<()> {
    ctx.defer().await?;
    match language_detection::detect_language(&text) {
        Ok(detected_language) => {
            log::info!("translate_text::detected_language::{:?}", detected_language);

            let mainnet_api_url = ctx.data().mainnet_api_url.clone();
            let translated_text = text2text(mainnet_api_url, &text, &detected_language, &target_language).await?;

            ctx.say(translated_text).await?;

            Ok(())
        }
        Err(error_message) => {
            log::error!("translate_text::language_detection::{:?}", error_message);
            ctx.say(error_message).await?;

            Ok(())
        }
    }
}

/// Translate this message.
#[poise::command(context_menu_command = "Translate", prefix_command)]
pub async fn translate_message(
    ctx: Context<'_>,
    #[description = "Message to translate (enter a link or ID)"]
    msg: serenity::Message,
) -> Result<()> {
    let interaction_id = ctx.id();
    let message_content = &msg.content;

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
            .content("Which language would you like to translate to?")
            .components(components)
            .ephemeral(true)
    };

    let ephemeral_reply = ctx.send(reply).await?;

    match language_detection::detect_language(message_content) {
        Ok(detected_language) => {
            log::info!("translate_message::detected_language::{:?}", detected_language);

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
                    let mainnet_api_url = ctx.data().mainnet_api_url.clone();
                    let translated_text = text2text(mainnet_api_url, message_content, &detected_language, &chosen_value).await?;
                    msg.reply(ctx, format!("{}", translated_text)).await?;
                    info!("translate_message::sent_reply");
                    ephemeral_reply.delete(ctx).await?;
                    component_interaction.delete_response(ctx).await?;
                }

            Ok(())
        },
        Err(err) => {
            msg.reply(ctx, err).await?;
            ephemeral_reply.delete(ctx).await?;
            Ok(())
        }
    }
}

/// Transcribes an audio file. Requires an audio file upload.
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
    let mainnet_api_url = ctx.data().mainnet_api_url.clone();
    let filename = file.filename;
    let url = file.url;
    let filesize = file.size;
    info!("audio_to_text::filename({})::filesize({})", &filename, filesize);
    fetch_file(url, &filename).await?;

    let mut file = tokio::fs::File::open(&filename).await?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).await?;
    let encoded_data = general_purpose::STANDARD.encode(&bytes);

	let body = SubnetPost {
		input: encoded_data,
		source_language,
		target_language,
		task_string: "speech2text".to_string(),
	};

	info!("audio_to_text::body::{:?}", body);

    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/translation", mainnet_api_url))
        .header("content-type", "application/json")
        .json(&body)
        .send().await?;

	info!("audio_to_text::post_response::{:?}", response);

    let response_text: String = response.text().await?;
    let value: serde_json::Value = serde_json::from_str(&response_text).unwrap();
    let json_str: String = value.as_str().unwrap().into();
    info!("audio_to_text::response_text::{:?}", &json_str);
	let decoded_bytes = general_purpose::STANDARD.decode(&json_str)?;
    let decoded_text = std::str::from_utf8(&decoded_bytes)?;
    info!("audio_to_text::decoded_text::{:?}", decoded_text);

    delete_file(&filename).await?;

    ctx.say(decoded_text).await?;

    Ok(())
}

async fn handle_reaction(
    ctx: &serenity::Context,
    reaction: &serenity::Reaction,
    data: &Data,
) -> Result<()> {
    let available_flags = get_available_flags();
    let available_languages = get_available_languages();
    let emoji = reaction.emoji.to_string();
    for (index, s) in available_flags.iter().enumerate() {
        if *s == emoji {
            log::info!("handle_reaction::match_found::{}", available_languages[index]);
            let message = reaction.message(ctx).await?;
            let message_content = &message.content;
            log::info!("handle_reaction::message_content::{}", message_content);
            match detect_language(&message_content) {
                Ok(detected_language) => {
                    let mainnet_api_url = data.mainnet_api_url.clone();
                    let translation = text2text(mainnet_api_url, &message_content, &detected_language, &available_languages[index].to_string()).await?;
                    message.reply(ctx, translation).await?;
                }
                Err(error_text) => {
                    message.reply(ctx, error_text).await?;
                }
            }
        }
    }
    Ok(())
}

/// Test database connection
#[poise::command(slash_command, prefix_command)]
async fn test_database(
    ctx: Context<'_>,
) -> Result<()> {
    let client: deadpool_postgres::Client = ctx.data().pool.get().await?;
    let _stmt = "SELECT version();";
    let stmt = client.prepare(&_stmt).await.unwrap();
    let result = client
        .query(&stmt, &[])
        .await?;
    log::info!("test_database::{:?}", result);
    Ok(())
}

async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<()> {
    match event {
        serenity::FullEvent::Ready { data_about_bot, .. } => {
            log::info!("Logged in as {}", data_about_bot.user.name);
        }
        serenity::FullEvent::ReactionAdd { add_reaction } => {
            handle_reaction(ctx, add_reaction, data).await?;
        }
        _ => {}
    }
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

    let config_ = Config::builder()
        .add_source(::config::Environment::default())
        .build()
        .unwrap();
    let config: BotConfig = config_.try_deserialize().expect("Config incorrect.");

    let pool = config.pg.create_pool(None, NoTls).unwrap();
    info!("init::postgres_pool::{:?}", pool.status());

    let token = config.discord_token;
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![
                audio_to_text(),
                translate_message(),
                translate_text(),
                supported_languages(),
                test_database(),
            ],
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    mainnet_api_url: config.mainnet_api_url,
                    pool
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;
    info!("Client initialized, starting bot.");
    client.unwrap().start().await.unwrap();
}
