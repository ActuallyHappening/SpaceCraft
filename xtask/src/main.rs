use clap::{Parser, Subcommand};

#[derive(Parser)] // requires `derive` feature
#[command(bin_name = "cargo xtask")]
#[command(author, version, about, long_about = None)]
enum Cli {
	Release(Release),
	Dev(Dev),
}

#[derive(clap::Args)]
struct Release {
	#[command(subcommand)]
	platform: Platform,
}

#[derive(clap::Args)]
struct Dev {
	#[command(subcommand)]
	platform: Platform,
}

#[derive(Subcommand)]
enum Platform {
	Windows,
	MacOS,
	Web,
}

fn main() {
	let _args = Cli::parse();
}
