use clap::ArgMatches;

pub async fn run(_cli_matches: &ArgMatches) {
    if let Err(e) = server::run().await {
        eprintln!("{e:?}");
    }
}
