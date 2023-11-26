use clap::{Parser, Subcommand};
use xtask::*;

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

	#[arg(long, short, default_value_t = get_osx_app_name())]
	app_name: String,
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

fn main() {
	let args = Cli::parse();

	match args {
		Cli::Release(Release { platform, bin_name, app_name }) => match platform {
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
