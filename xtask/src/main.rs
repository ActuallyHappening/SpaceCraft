use std::fs::{create_dir, create_dir_all, remove_dir_all, remove_file};
use semver::{Prerelease, BuildMetadata};
use tracing::*;

use clap::{Parser, Subcommand, ValueEnum, error};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use xtask::*;

#[derive(Parser, Debug)] // requires `derive` feature
#[command(bin_name = "cargo xtask")]
#[command(author, version, about, long_about = None)]
enum Cli {
	/// Builds and packages the application for release.
	Package(Package),

	/// Updates icons and ensures rustup has targets added.
	Prepare(Prepare),

	Release(Release),

	/// Updates dependencies like rustup update
	Update,
}

#[derive(clap::Args, Debug)]
struct Package {
	#[arg(long, default_value_t = get_bin_name())]
	bin_name: String,

	#[arg(long, default_value_t = get_osx_app_name())]
	app_name: String,

	/// Will ln -s the un-compressed package into applications.
	/// Only applicable for MacOS <-> MacOS builds.
	#[arg(long, default_value_t = false)]
	macos_link_into_applications: bool,

	/// Links in /Applications into the .dmg, so that the user can drag the app into /Applications.
	/// Only applicable for -> MacOS builds
	#[arg(long, default_value_t = true)]
	macos_link_for_bundle: bool,

	/// Will automatically call `open` on the package after building.
	/// Only applicable for MacOS builds
	#[arg(long, default_value_t = false)]
	macos_open: bool,

	#[command(subcommand)]
	platform: Platform,
}

#[derive(clap::Args, Debug)]
struct Prepare {
	#[command(subcommand)]
	platform: Platform,
}

#[derive(clap::Args, Debug)]
struct Release {
	#[arg(long, default_value_t = false)]
	all: bool,

	#[arg(long, default_value_t = false)]
	windows: bool,

	#[arg(long, default_value_t = false)]
	macos: bool,

	/// Manually specify the exact version
	#[arg(long, short)]
	version: Option<String>,

	/// Just bump the current dev-* patch version by one,
	/// e.g. 0.0.0-dev-1
	#[arg(long, default_value_t = false)]
	dev_patch: bool,

	/// Bump the version recorded in Cargo.toml and other places
	#[arg(long, default_value_t = true)]
	update: bool,

	title: String,

	#[arg(long, default_value_t = false)]
	proper_release: bool,
}

#[derive(Subcommand, Debug, Clone)]
enum Platform {
	Windows,
	#[command(name = "macos")]
	MacOS,
	// Web,
	// Linux,
}

fn main() {
	tracing_subscriber::fmt()
		.with_env_filter(
			EnvFilter::builder()
				.with_default_directive(LevelFilter::INFO.into())
				.parse_lossy("xtask=trace"),
		)
		.init();

	debug!("Initialized tracing");

	let cwd = std::env::current_dir().unwrap();
	let intended_dir = {
		let mut d = get_self_manifest_path();
		d.pop();
		d
	};
	if cwd != intended_dir {
		debug!("Changing CWD to {:?}", intended_dir);
		std::env::set_current_dir(intended_dir).unwrap();
	} else {
		trace!("In correct CWD");
	}

	trace!("About to parse CLI args ...");
	let args = Cli::parse();
	trace!("Parsed CLI args: {:?}", args);

	match args {
		Cli::Package(Package {
			platform,
			bin_name,
			app_name,
			macos_link_into_applications: link_into_applications,
			macos_link_for_bundle: link_for_bundle,
			macos_open: open,
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
				let bin_path = format!(
					"target/x86_64-pc-windows-gnu/release/{bin_name}.exe",
					bin_name = bin_name
				);
				assert!(Path::new(bin_path.as_str()).is_file());

				let release_folder = "release/windows";
				let release_dir = format!("{}/src", release_folder);
				if remove_dir_all(PathBuf::from(&release_dir)).is_ok() {
					info!("Removed old release {}", release_dir);
				}
				create_dir_all(&release_dir).unwrap();

				// copy assets, binary and eventually credits
				exec(
					"cp",
					["-r", "assets/", format!("{}/assets", release_dir).as_str()],
				);
				exec(
					"cp",
					[
						&bin_path,
						format!("{}/{}.exe", release_dir, bin_name).as_str(),
					],
				);

				// put into zip
				let version = get_current_version();
				let final_zip = format!("{} v{}.zip", app_name, version);
				// cwd into release_dir
				let original_cwd = std::env::current_dir().unwrap();
				std::env::set_current_dir(release_dir).unwrap();

				if PathBuf::from(&final_zip).exists() {
					info!("Removing old zip: rm \"{}\"", &final_zip);
					remove_file(&final_zip).unwrap();
				}
				exec("zip", ["-r", &final_zip, "."]);

				std::env::set_current_dir(original_cwd).unwrap();

				info!("Successfully packaged windows application: {}", final_zip);
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
				let package_folder = "release/macos/src";
				let package_dir = format!("{package_folder}/{app_name}.app",);
				if remove_dir_all(PathBuf::from(&package_folder)).is_ok() {
					println!("Removed old package");
				}
				create_dir_all(Path::new(&package_dir)).expect("Unable to create package directory");

				// copy assets, binary and eventually credits
				let assets_dir = format!("{}/Contents/MacOS/assets", &package_dir);
				create_dir_all(&assets_dir).unwrap();
				exec("cp", ["-r", "assets/", &assets_dir]);
				let final_bin_file = format!(
					"{}/Contents/MacOS/{bin_name}",
					&package_dir,
					bin_name = bin_name
				);
				exec("cp", [&bin_file, final_bin_file.as_str()]);
				exec("strip", [final_bin_file.as_str()]);

				// copy over contents in build/macos
				let build_dir = "build/macos.app";
				exec(
					"cp",
					[
						format!("{build_dir}/Contents/Info.plist").as_str(),
						format!("{package_dir}/Contents/Info.plist").as_str(),
					],
				);
				create_dir(format!("{package_dir}/Contents/Resources")).unwrap();
				exec(
					"cp",
					[
						format!("{build_dir}/Contents/Resources/AppIcon.icns").as_str(),
						format!("{package_dir}/Contents/Resources/AppIcon.icns").as_str(),
					],
				);

				if link_for_bundle {
					// ln -s /Applications into the bundle
					exec("ln", ["-s", "/Applications", &package_folder]);
				}

				// put into volume
				let version = get_current_version();
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
						// &bin_name,
						"-srcfolder",
						&package_folder,
						&final_dmg,
					],
				);

				// if link, ln -s into /Applications
				if link_into_applications {
					let app_link = format!("/Applications/{app_name}.app", app_name = app_name);
					if PathBuf::from(&app_link).is_symlink() || PathBuf::from(&app_link).is_file() {
						println!("Removing old app link: rm -rf \"{}\"", app_link);
						remove_file(&app_link).unwrap();
					}
					println!("Linking: ln -s \"{}\" \"{}\"", &package_dir, &app_link);
					exec("ln", ["-s", &package_dir, &app_link]);
				}

				if open {
					println!("Opening: open \"{}\"", package_dir);
					exec("open", [package_dir.as_str()]);
				}

				// eventually, code sign and notarize here
			}
		},
		Cli::Prepare(Prepare {
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

				// sort out icons
				let build_dir = "build/macos.app/Contents";
				let icon_dir = format!("{}/AppIcon.iconset", build_dir);
				if remove_dir_all(&icon_dir).is_ok() {
					println!("Removed old iconset dir");
				}
				create_dir(&icon_dir).unwrap();

				let base_icon = "assets/images/icon_1024x1024.png";
				assert!(Path::new(base_icon).exists());

				let sips = |size: u16| {
					exec(
						"sips",
						[
							"-z",
							size.to_string().as_str(),
							size.to_string().as_str(),
							base_icon,
							"--out",
							format!("{}/icon_{}x{}.png", icon_dir, size, size).as_str(),
						],
					);
				};
				let sips2 = |size: u16| {
					assert!(size > 16);
					exec(
						"sips",
						[
							"-z",
							size.to_string().as_str(),
							size.to_string().as_str(),
							base_icon,
							"--out",
							format!("{}/icon_{}x{}@2x.png", icon_dir, size / 2, size / 2).as_str(),
						],
					);
				};
				for size in [16, 32, 128, 256, 512].iter() {
					sips(*size);
				}
				for size in [32, 64, 256, 512].iter() {
					sips2(*size);
				}

				exec(
					"iconutil",
					[
						"-c",
						"icns",
						&icon_dir,
						"--output",
						format!("{}/Resources/AppIcon.icns", build_dir).as_str(),
					],
				);
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
		Cli::Release(Release {
			all,
			windows,
			macos,
			title,
			version,
			proper_release,
			dev_patch,
			update,
		}) => {
			if !all && !windows && !macos {
				error!("You must specify at least one platform to release, e.g. --macos or --all");
				std::process::exit(1);
			}

			let current_vers = get_current_version();
			let current_version = current_vers.parse::<semver::Version>().expect("Current version is not valid");

			let finalized_new_version;

			match (version, dev_patch) {
				(None, false) => {
					error!("Must provide either --version 0.1.2-dev.3 or --dev-patch");
					std::process::exit(1);
				}
				(Some(_), true) => {
					error!("Cannot provide both --version and --dev-patch");
					std::process::exit(1);
				}
				(Some(ver), false) => {
					let new_version = ver.parse::<semver::Version>().expect("New version is not valid");
					if new_version <= current_version {
						error!("Version {} is already the current version or less, please provide a version greater than {}", new_version, current_vers);
						std::process::exit(1);
					} else {
						finalized_new_version = format!("{}", new_version);
					}
				}
				(None, true) => {
					trace!("Incrementing dev patch version from {}", current_version);
					assert!(current_version.build.is_empty());
					if current_version.pre.is_empty() {
						trace!("Prerelease is empty, adding -dev.1");
						finalized_new_version = format!("{}-dev.1", current_version);
					} else {
						// extract last number after decimal place, increment by one
						let pre = current_version.pre.as_str();
						let num = pre.split('.').last().unwrap().parse::<u64>().unwrap();
						let mut pre = current_version.clone();
						pre.pre = Prerelease::EMPTY;
						finalized_new_version = format!("{}-dev.{}", pre, num + 1);
					}
				}
			}
			assert!(finalized_new_version.parse::<semver::Version>().is_ok());
			debug!("Finalized version for release: {}", finalized_new_version);

			if update {
				set_current_version(finalized_new_version);
			}

			let release_dir = "release/gh-releases";
		}
	}
}
