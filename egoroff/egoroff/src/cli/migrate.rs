use std::path::PathBuf;

use clap::ArgMatches;

pub async fn run(cli_matches: &ArgMatches)  { 
    let from_file = cli_matches.get_flag("file");
    let uri = cli_matches.get_one::<String>("uri").unwrap();

    let db_dir = cli_matches.get_one::<String>("dbpath").unwrap();
    let db_dir = PathBuf::from(db_dir).join(kernel::sqlite::DATABASE);
    let db_dir = db_dir.as_os_str().to_str().unwrap_or_default();
    migrate::run(uri, db_dir, from_file).await;
}