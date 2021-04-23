mod instance_data;
mod task_config;
mod zip;

use std::fs::{DirBuilder, ReadDir};
use std::io::ErrorKind;
use std::{fs, io};
use tracing::{debug, info};

pub use self::instance_data::InstanceData;
pub use self::task_config::TaskConfig;
pub use self::zip::zip_file;
use crate::baca::details::Language;
use crate::error::Error;
use crate::error::Result;

// todo: walk up dir tree until found
const BACA_DIR: &str = ".baca";
const INSTANCE_PATH: &str = ".baca/instance";
const TASK_PATH: &str = ".baca/task";

pub fn initialize() -> Result<()> {
    let baca_dir = check_if_initialized();

    if baca_dir.is_ok() {
        return Err(Error::WorkspaceAlreadyInitialized);
    }

    info!("Creating new {}", BACA_DIR);
    DirBuilder::new()
        .create(BACA_DIR)
        .map_err(as_config_create_error)?;

    info!("Baca directory created successfully.");
    Ok(())
}

pub fn save_instance(instance: &InstanceData) -> Result<()> {
    info!("Saving instance to the workspace.");
    let serialized = serde_json::to_string(instance).expect("Instance serialization error");
    debug!("Serialized: {}", serialized);

    fs::write(INSTANCE_PATH, serialized).map_err(as_config_write_error)?;
    Ok(())
}

pub fn read_instance() -> Result<InstanceData> {
    check_if_initialized()?;

    info!("Reading {}", INSTANCE_PATH);
    let serialized = fs::read_to_string(INSTANCE_PATH).map_err(as_config_read_error)?;
    debug!("Serialized: {}", serialized);

    let deserialized: InstanceData = serde_json::from_str(&serialized)?;
    debug!("Deserialized: {:?}", deserialized);
    Ok(deserialized)
}

pub fn read_task() -> Result<TaskConfig> {
    check_if_initialized()?;
    info!("Reading task from workspace.");
    let serialized = fs::read_to_string(TASK_PATH).map_err(as_task_read_error)?;
    debug!("Serialized: {}", serialized);

    let deserialized: TaskConfig = serde_json::from_str(&serialized)?;
    debug!("Deserialized: {:?}", deserialized);

    info!("Read task successfully.");
    Ok(deserialized)
}

pub fn save_task(task_id: &str, filepath: &str, to_zip: bool, language: Language) -> Result<()> {
    info!("Saving task info to {}.", TASK_PATH);

    let task = TaskConfig {
        id: task_id.to_string(),
        file: filepath.to_string(),
        language,
        to_zip,
    };
    let serialized = serde_json::to_string(&task)?;
    debug!("Serialized: {}", serialized);

    fs::write(TASK_PATH, serialized).map_err(as_task_write_error)?;
    info!("Saved task successfully.");
    Ok(())
}

pub fn remove_task() -> Result<()> {
    info!("Removing task from {}.", TASK_PATH);
    fs::remove_file(TASK_PATH).map_err(as_task_remove_error)
}

pub fn remove_workspace() -> Result<()> {
    info!("Removing Baca workspace.");
    fs::remove_dir_all(BACA_DIR).map_err(as_config_remove_error)
}

fn check_if_initialized() -> Result<ReadDir> {
    info!("Checking if {} exists.", BACA_DIR);
    fs::read_dir(BACA_DIR).map_err(as_not_init_error)
}

fn as_not_init_error(e: io::Error) -> Error {
    match e.kind() {
        ErrorKind::NotFound => Error::WorkspaceNotInitialized,
        _ => Error::OpeningWorkspaceError(e.into()),
    }
}

fn as_config_read_error(e: io::Error) -> Error {
    match e.kind() {
        ErrorKind::NotFound => Error::WorkspaceCorrupted,
        _ => Error::OpeningWorkspaceError(e.into()),
    }
}

fn as_config_write_error(e: io::Error) -> Error {
    Error::WritingWorkspaceError(e.into())
}

fn as_config_create_error(e: io::Error) -> Error {
    Error::CreatingWorkspaceError(e.into())
}

fn as_config_remove_error(e: io::Error) -> Error {
    Error::RemovingWorkspaceError(e.into())
}

fn as_task_remove_error(e: io::Error) -> Error {
    Error::RemovingTaskError(e.into())
}

fn as_task_read_error(e: io::Error) -> Error {
    Error::ReadingTaskError(e.into())
}

fn as_task_write_error(e: io::Error) -> Error {
    Error::ReadingTaskError(e.into())
}
