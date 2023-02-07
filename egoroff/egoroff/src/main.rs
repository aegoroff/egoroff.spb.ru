use clap::{arg, command, crate_name, ArgAction, Command};

mod cli;

#[tokio::main]
async fn main() {
    let cli = command!(crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand(Command::new(cli::VERSION_SUBCOMMAND).about(cli::VERSION_DESCRIPTION))
        .subcommand(
            Command::new(cli::MIGRATE_SUBCOMMAND)
                .about(cli::MIGRATE_DESCRIPTION)
                .arg(arg!(-u --uri <URI>).required(true).help("All posts URI"))
                .arg(
                    arg!(-d --dbpath <DBPATH>)
                        .required(true)
                        .help("Database directory path"),
                )
                .arg(
                    arg!(-f --file)
                        .required(false)
                        .action(ArgAction::SetTrue)
                        .help("Use files instead if remote resource"),
                ),
        )
        .subcommand(Command::new(cli::SERVER_SUBCOMMAND).about(cli::SERVER_DESCRIPTION))
        .arg_required_else_help(true)
        .disable_version_flag(true)
        .get_matches();

    if cli.subcommand_matches(cli::VERSION_SUBCOMMAND).is_some() {
        cli::version::run();
    } else if let Some(server_matches) = cli.subcommand_matches(cli::SERVER_SUBCOMMAND) {
        cli::server::run(server_matches).await;
    } else if let Some(migrate_matches) = cli.subcommand_matches(cli::MIGRATE_SUBCOMMAND) {
        cli::migrate::run(migrate_matches).await;
    }
}
