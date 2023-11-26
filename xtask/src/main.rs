use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

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

	#[arg(long, short, default_value_t = get_bin_name())]
	bin_name: String,
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
	#[command(name = "macos")]
	MacOS,
	// Web,
	// Linux,
}

fn get_cargo_path() -> String {
	// get cargo executable from env CARGO, and run it with str
	let cargo_exec_path = std::env::var("CARGO").unwrap();
	assert!(Path::new(&cargo_exec_path).is_file());
	cargo_exec_path
}

fn cargo_exec<'s>(args: impl IntoIterator<Item = &'s str>) {
	// get cargo executable from env CARGO, and run it with str
	let cargo_exec_path = get_cargo_path();
	exec(&cargo_exec_path, args);
}

fn exec<'a, 's>(exec_str: &'a str, args: impl IntoIterator<Item = &'s str>) -> String {
	let mut exec = std::process::Command::new(exec_str);
	exec.args(args);
	exec.stdout(Stdio::piped());

	let exec_output = exec.spawn().unwrap().wait_with_output().unwrap();
	assert!(exec_output.status.success());

	exec_output
		.stdout
		.into_iter()
		.map(|b| b as char)
		.collect::<String>()
}

fn exec_with_envs<'a, 's>(
	exec_str: &'a str,
	args: impl IntoIterator<Item = &'s str>,
	envs: impl IntoIterator<Item = (&'s str, &'s str)>,
) {
	let mut exec = std::process::Command::new(exec_str);
	exec.args(args);
	exec.envs(envs);
	let exec_output = exec.spawn().unwrap().wait().unwrap();
	assert!(exec_output.success());
}

fn get_bin_name() -> String {
	// parse Cargo.toml for bin name
	let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
	let cargo_toml: toml::Value = toml::from_str(&cargo_toml).unwrap();
	let bin_name = cargo_toml["package"]["name"].as_str().unwrap();
	bin_name.to_string()
}

#[cfg(target_os = "macos")]
fn get_sdk_root() -> PathBuf {
	let str = exec("xcrun", ["-sdk", "macosx", "--show-sdk-path"]);
	let str = str.trim();
	let path = Path::new(str);

	assert!(path.exists(), "SDK path {str} does not exist");

	PathBuf::from(path)
}

fn main() {
	let args = Cli::parse();

	match args {
		Cli::Release(Release { platform, bin_name }) => match platform {
			Platform::Windows => {
				cargo_exec([
					"build",
					"--release",
					"--target",
					"x86_64-pc-windows-gnu",
					"--no-default-features",
					"--features",
					"release",
				]);
				assert!(Path::new("target/x86_64-pc-windows-gnu/release/").is_dir());
				assert!(Path::new(
					format!("target/x86_64-pc-windows-gnu/release/{}.exe", bin_name).as_str()
				)
				.is_file());

				todo!("Package windows build");
			}
			Platform::MacOS => {
				let sdk_root = get_sdk_root();
				let sdk_root = sdk_root.to_str().unwrap();
				exec_with_envs(
					&get_cargo_path(),
					[
						"build",
						"--release",
						"--no-default-features",
						"--features",
						"release",
						"--target=aarch64-apple-darwin",
					],
					[("SDKROOT", sdk_root)],
				);
				let silicon_build = format!("target/aarch64-apple-darwin/release/{bin_name}");
				assert!(PathBuf::from(&silicon_build).is_file());

				exec_with_envs(
					&get_cargo_path(),
					[
						"build",
						"--release",
						"--no-default-features",
						"--features",
						"release",
						"--target=x86_64-apple-darwin",
					],
					[("SDKROOT", sdk_root)],
				);
				let intel_build = format!("target/x86_64-apple-darwin/release/{bin_name}");
				assert!(PathBuf::from(&intel_build).is_file());

				exec("lipo", [
					"-create",
					"-output",
					format!("target/release/{bin_name}").as_str(),
					&silicon_build,
					&intel_build,
				]);
			}
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
			Platform::MacOS => {
				exec("rustup", ["target", "add", "aarch64-apple-darwin"]);
				exec("rustup", ["target", "add", "x86_64-apple-darwin"]);
			}
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
