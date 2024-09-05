use anyhow::{bail, Context, Result};
use clap::Parser;
use env_logger::Env;
use log::{debug, error, info};
use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::process::Command;
use std::{fs, sync::mpsc};

/// Invoke processes based on incoming files in a directory.
#[derive(Debug, Parser)]
#[clap(name = "dirqd", version = "0.1.0", author = "Kenneth Wang Pedersen")]
pub struct DirQDArgs {
    /// Directory to watch
    directory: PathBuf,

    /// Command to invoke
    command: Vec<String>,

    /// Files must match this shell pattern
    #[arg(short = 'p', long, default_value = "*")]
    pattern: glob::Pattern,

    /// After successfully processing, move files here. Cannot be used with --delete.
    #[arg(short = 'P', long)]
    processed_queue: Option<PathBuf>,

    /// After successful processing, delete file. Cannot be used with --processed-queue.
    #[arg(long)]
    delete: bool,
    /// If invoking command fails, move files here. Cannot be used with --delete-on-error.
    #[arg(short = 'E', long)]
    error_queue: Option<PathBuf>,

    /// If invoking command fails, delete file. Cannot be used with --error-queue.
    #[arg(long)]
    delete_on_error: bool,
}

fn main() -> Result<()> {
    let env = Env::default().filter_or("DIRQD_LOG", "debug");

    env_logger::init_from_env(env);

    let args = DirQDArgs::parse();

    // Cross validation
    if (args.error_queue.is_none() && !args.delete_on_error)
        || (args.error_queue.is_some() && args.delete_on_error)
    {
        bail!("Either -E/--error-queue or --delete-on-error must be specified");
    }
    if (args.processed_queue.is_none() && !args.delete)
        || (args.processed_queue.is_some() && args.delete)
    {
        bail!("Either -P/--processed-queue or --delete must be specified");
    }

    info!("Directory: {:?}", args.directory);
    info!("Command: {:?}", args.command);
    info!("Pattern: {}", args.pattern);
    if args.delete {
        info!("Files will be deleted after successful processing")
    } else {
        info!(
            "Processed queue: {:?}",
            args.processed_queue.to_owned().unwrap()
        );
    }
    if args.delete_on_error {
        info!("Files will be deleted on error")
    } else {
        info!("Error queue: {:?}", args.error_queue.to_owned().unwrap());
    }

    let (tx, rx) = mpsc::channel();

    let mut watcher = RecommendedWatcher::new(tx, Config::default())?;

    watcher
        .watch(&args.directory, RecursiveMode::NonRecursive)
        .with_context(|| format!("Could not watch directory {:?}", args.directory))?;

    scan_dir(&args);

    for res in rx {
        match res {
            Ok(event) => {
                info!("Event: {:?}", event);
                if event.kind.is_create() || event.kind.is_modify() {
                    scan_dir(&args);
                }
            }
            Err(error) => error!("Error while watching file: {error}"),
        }
    }

    Ok(())
}

fn scan_dir(config: &DirQDArgs) {
    match fs::read_dir(&config.directory) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(entry) => {
                        if entry.file_type().map(|t| t.is_file()).unwrap_or(false) {
                            handle_file(&entry, config)
                        }
                    }
                    Err(e) => error!("Error on directory entry: {e}"),
                }
            }
        }
        Err(e) => error!("Error while enumerating directory: {e}"),
    }
}

fn handle_file(entry: &fs::DirEntry, config: &DirQDArgs) {
    let entry_path_buf = entry.path().clone();
    let entry_path_opt = entry_path_buf.to_str();

    if entry_path_opt.is_none() {
        error!("Ignoring invalid path: {:?}", entry);
        return;
    }

    let entry_path = entry_path_opt.unwrap();

    if !config.pattern.matches(entry_path) {
        debug!("Ignoring {entry_path} because it doesn't match pattern");
        return;
    }

    let mut command_args = config.command.clone();
    command_args.push(entry_path.to_owned());

    info!("Executing: {:?}", command_args);

    let executable = command_args.first().unwrap();

    let mut command = Command::new(executable);
    command.args(command_args.iter().skip(1));

    match command.status() {
        Ok(status) => handle_ok(&status, entry, config),
        Err(error) => handle_error(&error, entry, config),
    }
}

fn handle_ok(status: &std::process::ExitStatus, entry: &fs::DirEntry, config: &DirQDArgs) {
    if status.success() {
        debug!("Process executed successfully");
        process_file(entry, &config.processed_queue);
    } else {
        let error_message = format!("Exited with status {:?}", status);
        let err = std::io::Error::new(std::io::ErrorKind::Other, error_message);
        handle_error(&err, entry, config);
    }
}

fn handle_error(error: &std::io::Error, entry: &fs::DirEntry, config: &DirQDArgs) {
    error!("While executing command for entry {:?}: {}", entry, error);
    process_file(entry, &config.error_queue);
}

fn process_file(entry: &fs::DirEntry, destination: &Option<PathBuf>) {
    match destination {
        Some(dir) => {
            let source_path = entry.path();
            let dest_path = dir.join(entry.file_name());
            debug!("Moving {:?} to {:?}", source_path, dest_path);
            match std::fs::rename(&source_path, &dest_path) {
                Ok(_) => (),
                Err(err) => error!(
                    "Error {} while renaming {:?} to {:?}",
                    err, source_path, dest_path
                ),
            }
        }
        None => {
            let source_path = entry.path();
            debug!("Deleting {:?}", source_path);
            match std::fs::remove_file(&source_path) {
                Ok(_) => (),
                Err(err) => error!("Error {} while deleting {:?}", err, source_path),
            }
        }
    }
}
