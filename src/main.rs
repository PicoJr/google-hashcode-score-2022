#[macro_use]
extern crate clap;
extern crate anyhow;
extern crate fxhash;
extern crate serde;

use crate::parser::{parse_input, parse_output};
use crate::score::{
    compute_score_precomputed, decode_precomputed, encode_precomputed, precompute_from_input,
    PreComputed, Score,
};
use log::{debug, info, warn};
use num_format::{Locale, ToFormattedString};
use serde::Serialize;
use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use warp::Filter;

mod cli;
mod data;
mod parser;
mod score;

#[derive(Serialize)]
struct Response {
    score: Score,
    valid: bool,
    message: Option<String>,
}

fn score_fn(
    body: &bytes::Bytes,
    precomputed: &PreComputed,
    disable_checks: bool,
) -> anyhow::Result<Score> {
    let body_string = String::from_utf8(body.to_vec())?;
    let output_data = parse_output(&body_string)?;
    let score = compute_score_precomputed(precomputed, &output_data, disable_checks)?;
    info!("score {}", score.to_formatted_string(&Locale::en));
    Ok(score)
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // cf https://crates.io/crates/env_logger
    env_logger::init();

    // parse command line arguments
    let matches = cli::get_command().get_matches();
    let input_file_paths = matches.values_of("input").expect("input file compulsory");

    let disable_checks = matches.is_present("disable-checks");
    if disable_checks {
        warn!("checks are disabled, score may be overestimated if output files are incorrect.")
    }
    let generate_cache_files = matches.is_present("cache");
    if generate_cache_files {
        warn!("cache file will be generated, expect slight performance degradation for this run.")
    }

    let precomputed_files: anyhow::Result<Vec<PreComputed>> = input_file_paths
        .map(|input_file_path| {
            let path = PathBuf::from_str(input_file_path)?;
            if path.extension() == Some(&OsString::from_str("bin").unwrap()) {
                decode_precomputed(&path)
            } else {
                let input_content = read_to_string(&path)?;
                info!("parsing {}", input_file_path);
                let input_data = parse_input(&input_content)?;
                debug!("{:?}", input_data);
                let precomputed = precompute_from_input(&input_data);
                if generate_cache_files {
                    let dump_path = PathBuf::from(&path).with_extension("bin");
                    encode_precomputed(&precomputed, &dump_path)?;
                }
                Ok(precomputed)
            }
        })
        .collect();
    let precomputed_files = precomputed_files?;

    let precomputed_files_arc = Arc::new(precomputed_files);

    let submit = warp::post()
        .and(warp::path("score"))
        .and(warp::path::param::<usize>())
        .and(warp::body::bytes())
        .and(warp::body::content_length_limit(1024 * 1024))
        .map(move |submission_id: usize, body: bytes::Bytes| {
            let precomputed_files = precomputed_files_arc.clone();

            if let Some(precomputed) = precomputed_files.get(submission_id) {
                match score_fn(&body, precomputed, disable_checks) {
                    Ok(score) => warp::reply::json(&Response {
                        score,
                        valid: true,
                        message: None,
                    }),
                    Err(err) => warp::reply::json(&Response {
                        score: 0,
                        valid: false,
                        message: Some(err.to_string()),
                    }),
                }
            } else {
                warp::reply::json(&Response {
                    score: 0,
                    valid: false,
                    message: Some(format!(
                        "invalid id: {} (should be >= 0 and < {})",
                        submission_id,
                        precomputed_files.len()
                    )),
                })
            }
        });

    let addr = ([127, 0, 0, 1], 3000);

    println!("Listening on http://{:?}", addr);

    warp::serve(submit).run(addr).await;

    Ok(())
}
