use clap::{Arg, Command};

pub fn get_command() -> Command<'static> {
    Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Google Hashcode Score Calculator")
        .arg(
            Arg::new("input")
                .help("input file path")
                .multiple_values(true)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("disable-checks")
                .long("--disable-checks")
                .help("disable checks (contributors level)")
                .required(false)
                .takes_value(false),
        )
        .arg(
            Arg::new("cache")
                .long("--cache")
                .help("generate cache files (.bin files)")
                .required(false)
                .takes_value(false),
        )
}
