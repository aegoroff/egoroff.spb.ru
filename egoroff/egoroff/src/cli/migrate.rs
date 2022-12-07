use clap::ArgMatches;

pub async fn run(cli_matches: &ArgMatches)  { 
    let uri = cli_matches.get_one::<String>("uri").unwrap();
    migrate::run(uri).await;
}