#![allow(unused)]
use std::fmt::Display;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct GithubPushEvent {
    #[serde(rename = "ref")]
    pub ref_name:   String,
    pub repository: Repository,
    pub commits:    Vec<Commit>,
    pub sender:     Sender,
}

impl Display for GithubPushEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "由 {} 推送到 {} ;\n提交内容: \n{}",
            self.sender.login,
            self.ref_name,
            self.commits.iter().map(|c| c.message.clone()).collect::<Vec<_>>().join("\n")
        )
    }
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub name:      String,
    pub full_name: String,
    pub html_url:  String,
}

#[derive(Deserialize, Debug)]
pub struct Commit {
    pub id:      String,
    pub message: String,
    pub author:  Author,
    pub url:     String,
}

#[derive(Deserialize, Debug)]
pub struct Author {
    pub name:  String,
    pub email: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct Sender {
    pub login: String,
}
