use std::fs::{create_dir_all, remove_dir_all, remove_file};

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
		Cli::Release(Release {
			platform,
			bin_name,
			app_name,
		}) => match platform {
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
			#[cfg(not(target_os = "macos"))]
			Platform::MacOS => {
				unimplemented!("Building for MacOS from a non-macos platform is not supported. Please run this command from a macos machine.")
			}
			#[cfg(target_os = "macos")]
			Platform::MacOS => {
				// macos packaging

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

				let bin_file = format!("target/release/{bin_name}", bin_name = bin_name);
				exec(
					"lipo",
					[
						"-create",
						"-output",
						&bin_file,
						&silicon_build,
						&intel_build,
					],
				);

				// prepare package_path
				let dir = format!(
					"release/macos/{app_name}.app",
					app_name = app_name.replace(' ', "")
				);
				let package_path = PathBuf::from(&dir);
				if remove_dir_all(&package_path).is_ok() {
					println!("Removed old package");
				}
				create_dir_all(&package_path).expect("Unable to create package directory");

				// copy assets, binary and eventually credits
				let assets_dir = format!("{}/Contents/MacOS/assets", &dir);
				create_dir_all(&assets_dir).unwrap();
				exec(
					"cp",
					[
						"-r",
						"assets/",
						&assets_dir,
					],
				);
				let final_bin_file = format!("{}/Contents/MacOS/{bin_name}", &dir, bin_name = bin_name);
				exec("cp", [&bin_file, final_bin_file.as_str()]);
				exec("strip", [final_bin_file.as_str()]);
				// todo: copy over icons from build/macos

				// put into volume
				let version = get_version_string();
				let final_dmg = format!("release/macos/{app_name} v{version}.dmg");
				if PathBuf::from(&final_dmg).is_file() {
					println!("Removing old dmg: {}", final_dmg);
					remove_file(&final_dmg).unwrap();
				}
				exec(
					"hdiutil",
					[
						"create",
						"-fs",
						"HFS+",
						"-volname",
						&app_name,
						"-srcfolder",
						&dir,
						&final_dmg,
					],
				);

				// eventually, code sign and notarize here
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
