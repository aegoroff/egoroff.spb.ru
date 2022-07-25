use clap::{App, SubCommand};

mod cli;

#[tokio::main]
async fn main() {
    let cli = App::new(clap::crate_name!())
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .subcommand(SubCommand::with_name(cli::VERSION_SUBCOMMAND).about(cli::VERSION_DESCRIPTION))
        .subcommand(
            SubCommand::with_name(cli::SERVER_SUBCOMMAND).about(cli::SERVER_DESCRIPTION),
        )
        .arg_required_else_help(true)
        .disable_version_flag(true)
        .get_matches();

    if let Some(_) = cli.subcommand_matches(cli::VERSION_SUBCOMMAND) {
        cli::version::run();
    } else if let Some(server_matches) = cli.subcommand_matches(cli::SERVER_SUBCOMMAND) {
        cli::server::run(server_matches).await;
    }
}
