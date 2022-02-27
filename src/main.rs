#[macro_use]
extern crate clap;
extern crate anyhow;
extern crate fxhash;

use crate::parser::{parse_input, parse_output};
use crate::score::{
    compute_score_precomputed, decode_precomputed, encode_precomputed, precompute_from_input, Score,
};
use anyhow::bail;
use log::{debug, info, warn};
use num_format::{Locale, ToFormattedString};
use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;

mod cli;
mod data;
mod parser;
mod score;

fn main() -> anyhow::Result<()> {
    // cf https://crates.io/crates/env_logger
    env_logger::init();

    // parse command line arguments
    let matches = cli::get_command().get_matches();
    let input_files = matches.values_of("input").expect("input files compulsory");
    let output_files = matches
        .values_of("output")
        .expect("output files compulsory");
    let many = input_files.len() > 1;
    if input_files.len() != output_files.len() {
        bail!(
            "{} output files provided but expected {}",
            output_files.len(),
            input_files.len()
        );
    }
    let disable_checks = matches.is_present("disable-checks");
    if disable_checks {
        warn!("checks are disabled, score may be overestimated if output files are incorrect.")
    }
    let generate_cache_files = matches.is_present("cache");
    if generate_cache_files {
        warn!("cache file will be generated, expect slight performance degradation for this run.")
    }
    let mut total_score: Score = 0;
    let input_output_files = input_files.zip(output_files);
    for (input_file_path, output_file_path) in input_output_files {
        let path = PathBuf::from_str(output_file_path)?;
        let output_content = read_to_string(path)?;
        info!("parsing {}", output_file_path);
        // parsing output first since it is most likely to fail
        let output_data = parse_output(&output_content)?;
        debug!("{:?}", output_data);

        let path = PathBuf::from_str(input_file_path)?;
        let mut precomputed = if path.extension() == Some(&OsString::from_str("bin").unwrap()) {
            decode_precomputed(&path)?
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
            precomputed
        };

        let score = compute_score_precomputed(&mut precomputed, &output_data, disable_checks)?;
        total_score += score;
        let formatted_score = score.to_formatted_string(&Locale::en);
        println!("{} score: {}", output_file_path, formatted_score);
    }
    if many {
        let formatted_score = total_score.to_formatted_string(&Locale::en);
        println!("total score: {}", formatted_score);
    }
    Ok(())
}
