use std::path::Path;

use clap::{Parser, Subcommand};

#[derive(Parser)] // requires `derive` feature
#[command(bin_name = "cargo xtask")]
#[command(author, version, about, long_about = None)]
enum Cli {
	Release(Release),
	Dev(Dev),
	Setup(Setup),
	Update,
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

#[derive(clap::Args)]
struct Setup {
	#[command(subcommand)]
	platform: Platform,
	// #[arg(long, short)]
	// user_name: String,
}

#[derive(Subcommand)]
enum Platform {
	Windows,
	MacOS,
	Web,
}

fn cargo_exec<'s>(args: impl IntoIterator<Item = &'s str>) {
	// get cargo executable from env CARGO, and run it with str
	let cargo_exec_path = std::env::var("CARGO").unwrap();
	exec(&cargo_exec_path, args);
}

fn exec<'a, 's>(exec_str: &'a str, args: impl IntoIterator<Item = &'s str>) {
	let mut exec = std::process::Command::new(exec_str);
	exec.args(args);
	let exec_output = exec.spawn().unwrap().wait().unwrap();
	assert!(exec_output.success())
}

fn get_bin_name() -> String {
	// parse Cargo.toml for bin name
	let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
	let cargo_toml: toml::Value = toml::from_str(&cargo_toml).unwrap();
	let bin_name = cargo_toml["package"]["name"].as_str().unwrap();
	bin_name.to_string()
}

fn main() {
	let args = Cli::parse();

	match args {
		Cli::Release(Release { platform }) => match platform {
			Platform::Windows => {
				cargo_exec(["build", "--release", "--target", "x86_64-pc-windows-gnu", "--no-default-features", "--features", "release"]);
				assert!(Path::new("target/x86_64-pc-windows-gnu/release/").is_dir());
				let bin_name = get_bin_name();
				assert!(Path::new(format!("target/x86_64-pc-windows-gnu/release/{}.exe", bin_name).as_str()).is_file());

				todo!("Package windows build");
			}
			_ => todo!(),
		},
		Cli::Setup(Setup {
			platform,
			// user_name,
		}) => match platform {
			Platform::Windows => {
				// exec("rustup", ["target", "add", "x86_64-pc-windows-msvc"]);
				exec("rustup", ["target", "add", "x86_64-pc-windows-gnu"]);
				// cargo_exec(["install", "xwin"]);
				// exec(
				// 	"xwin",
				// 	[
				// 		"--accept-license",
				// 		"splat",
				// 		"--disable-symlinks",
				// 		"--output",
				// 		format!("/Users/{}/.xwin", user_name).as_str(),
				// 	],
				// );
				#[cfg(target_os = "macos")]
				exec("brew", ["install", "llvm"]);
				#[cfg(target_os = "macos")]
				exec("brew", ["install", "mingw-w64"]);
			}
			_ => todo!(),
		},
		Cli::Update => {
			cargo_exec(["update"]);
			exec("rustup", ["update"]);
			#[cfg(target_os = "macos")]
			exec("brew", ["update"]);
			#[cfg(target_os = "macos")]
			exec("brew", ["upgrade"]);
		}
		_ => todo!(),
	}
}
