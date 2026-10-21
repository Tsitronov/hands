
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
	pub id: String,
	pub title: String,
	pub steps: Vec<Step>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
	pub id: u32,
	pub title: String,
	pub image: String,
	pub video: Option<String>,
	pub audio: Option<String>
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]

pub enum HintLevel {
	Video,
	Highlighted,
	PictureOnly,
	Independent
}

impl HintLevel {
	pub fn less_help(self) -> Self {
        match self {
            HintLevel::Video => HintLevel::Highlighted,
            HintLevel::Highlighted => HintLevel::PictureOnly,
            HintLevel::PictureOnly | HintLevel::Independent => HintLevel::Independent,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            HintLevel::Video => "video",
            HintLevel::Highlighted => "highlighted",
            HintLevel::PictureOnly => "picture_only",
            HintLevel::Independent => "independent",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "video" => Some(HintLevel::Video),
            "highlighted" => Some(HintLevel::Highlighted),
            "picture_only" => Some(HintLevel::PictureOnly),
            "independent" => Some(HintLevel::Independent),
            _ => None,
        }
    }
}


pub fn load_task(json: &str) -> Result<Task, serde_json::Error> {
    serde_json::from_str(json)
}
