use clap::{Arg, Command};

pub fn get_command() -> Command<'static> {
    Command::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .about("Google Hashcode Score Calculator")
        .arg(
            Arg::new("input")
                .help("input file paths")
                .multiple_values(true)
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .help("output file paths (one for each input provided file)")
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
}
