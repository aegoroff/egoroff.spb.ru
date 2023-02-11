pub mod migrate;
pub mod server;
pub mod version;

pub const SERVER_SUBCOMMAND: &str = "server";
pub const SERVER_DESCRIPTION: &str = "Run the server";

pub const VERSION_SUBCOMMAND: &str = "version";
pub const VERSION_DESCRIPTION: &str = "Display the version and build information";

pub const MIGRATE_SUBCOMMAND: &str = "migrate";

#[cfg(feature = "migrating")]
pub const MIGRATE_DESCRIPTION: &str = "Migrate data from Google datastore";
