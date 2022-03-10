#[macro_use]
extern crate clap;
extern crate anyhow;
extern crate fxhash;

use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};

use crate::parser::{parse_input, parse_output};
use crate::score::{
    compute_score_precomputed, decode_precomputed, encode_precomputed, precompute_from_input,
    PreComputed,
};
use log::{debug, info, warn};
use num_format::{Locale, ToFormattedString};
use std::ffi::OsString;
use std::fs::read_to_string;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;

mod cli;
mod data;
mod parser;
mod score;

async fn score_fn(
    req: Request<Body>,
    precomputed: Arc<PreComputed>,
    disable_checks: bool,
) -> anyhow::Result<Response<Body>> {
    debug!("req: {:?}", req);
    let body_bytes = hyper::body::to_bytes(req.into_body()).await?;

    let body_string = String::from_utf8(body_bytes.to_vec())?;
    let output_data = parse_output(&body_string)?;

    let score = compute_score_precomputed(&precomputed, &output_data, disable_checks)?;

    info!("score {}", score.to_formatted_string(&Locale::en));
    Ok(Response::new(Body::from(format!("score: {}", score))))
}

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // cf https://crates.io/crates/env_logger
    env_logger::init();

    // parse command line arguments
    let matches = cli::get_command().get_matches();
    let input_file_path = matches.value_of("input").expect("input file compulsory");

    let disable_checks = matches.is_present("disable-checks");
    if disable_checks {
        warn!("checks are disabled, score may be overestimated if output files are incorrect.")
    }
    let generate_cache_files = matches.is_present("cache");
    if generate_cache_files {
        warn!("cache file will be generated, expect slight performance degradation for this run.")
    }

    let path = PathBuf::from_str(input_file_path)?;
    let precomputed = if path.extension() == Some(&OsString::from_str("bin").unwrap()) {
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

    let precomputed_arc = Arc::new(precomputed);

    // For every connection, we must make a `Service` to handle all
    // incoming HTTP requests on said connection.
    let make_svc = make_service_fn(move |_| {
        let precomputed = precomputed_arc.clone();

        // This is the `Service` that will handle the connection.
        // `service_fn` is a helper to convert a function that
        // returns a Response into a `Service`.
        async move {
            Ok::<_, anyhow::Error>(service_fn(move |req| {
                score_fn(req, precomputed.clone(), disable_checks)
            }))
        }
    });

    let addr = ([127, 0, 0, 1], 3000).into();

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    server.await?;

    Ok(())
}
