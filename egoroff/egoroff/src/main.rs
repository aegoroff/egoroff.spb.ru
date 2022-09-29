use clap::{Command, command, crate_name};

mod cli;

#[tokio::main]
async fn main() {
    let cli = command!(crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand(Command::new(cli::VERSION_SUBCOMMAND).about(cli::VERSION_DESCRIPTION))
        .subcommand(
            Command::new(cli::SERVER_SUBCOMMAND).about(cli::SERVER_DESCRIPTION),
        )
        .arg_required_else_help(true)
        .disable_version_flag(true)
        .get_matches();

    if cli.subcommand_matches(cli::VERSION_SUBCOMMAND).is_some() {
        cli::version::run();
    } else if let Some(server_matches) = cli.subcommand_matches(cli::SERVER_SUBCOMMAND) {
        cli::server::run(server_matches).await;
    }
}
