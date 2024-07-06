//! Project

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Project {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub body: String,
    pub status: String,
    pub requested_status: String,
    pub additional_categories: Vec<String>,
    pub issues_url: String,
    pub source_url: String,
    pub wiki_url: String,
    pub discord_url: String,
    pub donation_urls: Vec<DonationUrl>,
    pub project_type: String,
    pub downloads: i64,
    pub icon_url: String,
    pub color: i64,
    pub thread_id: String,
    pub monetization_status: String,
    pub id: String,
    pub team: String,
    pub body_url: Option<serde_json::Value>,
    pub moderator_message: Option<serde_json::Value>,
    pub published: String,
    pub updated: String,
    pub approved: String,
    pub queued: String,
    pub followers: i64,
    pub license: License,
    pub versions: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub gallery: Vec<Gallery>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DonationUrl {
    pub id: String,
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Gallery {
    pub url: String,
    pub featured: bool,
    pub title: String,
    pub description: String,
    pub created: String,
    pub ordering: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: String,
}
