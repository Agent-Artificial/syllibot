use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetPost {
	pub input: String,
	pub source_language: String,
	pub target_language: String,
	pub task_string: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubnetPostData {
	pub data: SubnetPost,
}

pub struct Data {} // User data, which is stored and accessible in all command invocations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

