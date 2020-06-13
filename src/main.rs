use clap::{App, Arg, SubCommand};

use snafu::Snafu;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Could not find CSRF token needed to perform request"))]
    CsrfTokenNotFound,
    #[snafu(display("Couldn't execute HTTP request: {}", source))]
    Execute {
        source: reqwest::Error,
    },
    #[snafu(display("Received invalid HTTP response: {}", source))]
    InvalidText {
        source: reqwest::Error,
    },
    #[snafu(display("Received invalid JSON: {}", source))]
    InvalidJson {
        source: serde_json::Error,
    }
}

pub mod saime;

fn main() {
    let matches = App::new("saimexploit")
        .version("0.1.0")
        .author("Samael <me@jeandudey.tech>")
        .about("Does wonderful things to know about people")
        .arg(Arg::with_name("verbose"))
        .subcommand(SubCommand::with_name("saime")
                    .about("SAIME interesting features")
                    .subcommand(SubCommand::with_name("get")
                                .arg(Arg::with_name("cedula")
                                     .takes_value(true)
                                     .value_name("CI")
                                     .required(true)
                                     .help("C.I. to get information about"))
                                .arg(Arg::with_name("save")
                                     .short("s")
                                     .long("save")
                                     .help("Save information if it's new"))))
        .get_matches();

    if let Some(saime_matches) = matches.subcommand_matches("saime") {
        if let Some(get_matches) = saime_matches.subcommand_matches("get") {
            if let Some(ci) = get_matches.value_of("cedula") {
                let json = saime::get(ci).unwrap();
                println!("{:?}", json);
            }
        }
    }
}
