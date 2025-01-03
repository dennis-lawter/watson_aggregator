use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::anyhow;
use anyhow::Result;
use chrono::Duration;
use chrono::{DateTime, Utc};
use clap::Parser;
use colored::Colorize;
use serde::Deserialize;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[arg(required = true, value_name = "FILE")]
    inputs: Vec<String>,
}
impl Cli {
    fn get_file_map(&self) -> Result<HashMap<String, PathBuf>> {
        let mut map = HashMap::new();

        for file in &self.inputs {
            let pos = file
                .find(":")
                .ok_or(anyhow!("File list must be in the format username:path"))?;
            let (id, path) = file.split_at(pos);
            let path = path
                .strip_prefix(":")
                .ok_or(anyhow!("File list must be in the format username:path"))?;
            map.insert(id.to_owned(), PathBuf::from_str(path)?);
        }

        Ok(map)
    }
}

#[derive(Deserialize, Debug)]
struct FrameRaw {
    start: i64,
    end: i64,
    project: String,
    id: String,
    tags: Vec<String>,
    _last_updated: i64,
}

#[derive(Debug)]
struct FrameClean {
    _id: String,
    start: DateTime<Utc>,
    end: DateTime<Utc>,
    _project: String,
    _tags: Vec<String>,
}
impl FrameClean {
    fn from_raw(raw: &FrameRaw) -> Option<Self> {
        Some(Self {
            _id: raw.id.clone(),
            start: DateTime::from_timestamp(raw.start, 0)?,
            end: DateTime::from_timestamp(raw.end, 0)?,
            _project: raw.project.clone(),
            _tags: raw.tags.clone(),
        })
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let file_map = cli.get_file_map()?;
    let mut report: HashMap<String, Duration> = HashMap::new();
    for (id, file) in file_map {
        let mut duration = Duration::zero();
        let raw = std::fs::read_to_string(file)?;
        let frames_raw: Vec<FrameRaw> = serde_json::from_str(&raw)?;
        for raw in frames_raw {
            let clean = FrameClean::from_raw(&raw)
                .ok_or(anyhow!("Frame could not be cleaned, likely a date error."))?;
            let frame_duration = clean.end - clean.start;
            duration += frame_duration;
        }
        report.insert(id, duration);
    }
    let mut total_seconds = 0;
    for (_, duration) in &report {
        total_seconds += duration.num_seconds();
    }
    // println!("==================================================");
    for (id, duration) in &report {
        println!("{:<50}", id.on_red().bold());
        let h = duration.num_hours();
        let m = duration.num_minutes() % 60;
        let s = duration.num_seconds() % 60;
        println!("{h}h {m}m {s}s");
        let p = duration.num_seconds() as f64 / total_seconds as f64;
        println!("{:.2}%", p * 100.0);
        // println!("==================================================");
        println!("");
    }

    Ok(())
}
