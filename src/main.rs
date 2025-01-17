#[macro_use]
extern crate clap;

use crate::update::{GithubReleases, UpdateCheckTimestamp, UpdateChecker, UpdateStatus};
use crate::workspace::{ConfigObject, WorkspaceDir};
use api::baca_service::BacaService;
use clap::{App, AppSettings, ArgMatches};
use colored::Colorize;
use std::env;
use tracing::{error, info, warn, Level};

mod api;
mod command;
mod error;
mod log;
mod model;
mod parse;
mod update;
mod workspace;

fn main() {
    let yaml = load_yaml!("cli.yml");
    let app = App::from_yaml(yaml)
        .version(env!("CARGO_PKG_VERSION"))
        .setting(AppSettings::ArgRequiredElseHelp);
    let matches = app.get_matches();
    let workspace = WorkspaceDir::new();
    let baca_api = BacaService::default();

    set_logging_level(&matches);
    check_for_updates(&workspace, &matches);

    if let (command, Some(sub_matches)) = matches.subcommand() {
        if let Err(e) = command::execute(&workspace, &baca_api, command, sub_matches) {
            error!("{:?}", e);
            println!("{}", format!("{}", e).bright_red());
        }
    }
}

fn set_logging_level(matches: &ArgMatches) {
    let verbose_matches = matches.occurrences_of("verbose");

    let log_level = match verbose_matches {
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };

    if verbose_matches != 0 {
        log::init_logging(log_level);
    }
}

fn check_for_updates(workspace: &WorkspaceDir, matches: &ArgMatches) {
    if matches.is_present("noupdate") {
        info!("Update check disabled.");
        return;
    }

    let now = UpdateCheckTimestamp::now();
    let last_check = UpdateCheckTimestamp::read_config(workspace).unwrap();

    if matches.is_present("force-update") || last_check.is_expired(&now) {
        let updates = fetch_updates();

        if let Err(e) = updates {
            error!("Error checking for updates: {}", e);
            return;
        }

        match updates.unwrap() {
            UpdateStatus::NoUpdates => {
                info!("No updates available.");

                now.save_config(workspace)
                    .unwrap_or_else(|e| warn!("Error saving last update check timestamp: {}", e));
            }
            UpdateStatus::Update(new_rel) => {
                println!(
                    "{}",
                    format!(
                        "New version {} is available!!\nDownload at {}",
                        new_rel.version, new_rel.link
                    )
                    .bright_yellow()
                )
            }
        }
    }
}

fn fetch_updates() -> error::Result<UpdateStatus> {
    let owner = env::var("GITHUB_USER").unwrap_or_else(|_| "hjaremko".to_string());
    let repo = env::var("GITHUB_REPO").unwrap_or_else(|_| "baca-cli".to_string());

    let gh_service = GithubReleases::new(&owner, &repo);
    let checker = UpdateChecker::new(gh_service, update::CURRENT_VERSION);
    checker.check_for_updates()
}
