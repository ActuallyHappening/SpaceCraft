use camino::{Utf8Path, Utf8PathBuf};
use semver::{BuildMetadata, Prerelease};
use std::fs::{create_dir, create_dir_all, remove_dir_all, remove_file};
use tracing::*;

use clap::{error, Args, Parser, Subcommand, ValueEnum};
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use xtask::*;

// #[derive(Parser, Debug)] // requires `derive` feature
// #[command(bin_name = "cargo xtask")]
// #[command(author, version, about, long_about = None)]
// enum Cli {
// 	/// Builds and packages the application for release.
// 	Package(Package),

// 	/// Updates icons and ensures rustup has targets added.
// 	Prepare(Prepare),

// 	Release(Release),

// 	/// Updates dependencies like rustup update
// 	Update,
// }

#[derive(Parser, Debug)]
#[command(bin_name = "cargo xtask")]
#[command(author, version, about, long_about = None)]
struct Cli {
	#[arg(long, default_value_t = get_default_release_dir())]
	target_release_dir: Utf8PathBuf,

	#[arg(long, default_value_t = get_default_build_dir())]
	static_build_dir: Utf8PathBuf,

	#[command(subcommand)]
	command: Commands,
}

// #[derive(clap::Args, Debug)]
// struct Package {

// 	#[command(subcommand)]
// 	platform: Platform,
// }

#[derive(Subcommand, Debug)]
enum Commands {
	Package {
		#[arg(long, default_value_t = get_default_bin_name())]
		bin_name: String,

		#[arg(long, default_value_t = get_default_osx_app_name())]
		app_name: String,

		#[arg(long, default_value_t = true)]
		output_final_path: bool,

		#[command(subcommand)]
		platform: Package,
	},
}

/// Build and package the application, ready for release
#[derive(Subcommand, Debug)]
enum Package {
	#[cfg(target_os = "macos")]
	Macos {
		/// Will ln -s the un-compressed package into applications.
		/// Only applicable for MacOS <-> MacOS builds.
		#[arg(long, default_value_t = false)]
		link_into_applications: bool,

		/// Links in /Applications into the .dmg, so that the user can drag the app into /Applications.
		/// Only applicable for -> MacOS builds
		#[arg(long, default_value_t = true)]
		link_for_bundle: bool,

		/// Will automatically call `open` on the package after building.
		/// Only applicable for MacOS builds
		#[arg(long, default_value_t = false)]
		open: bool,
	},

	Windows,
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

	let release_dir = args.target_release_dir;
	let build_dir = args.static_build_dir;

	match args.command {
		Commands::Package {
			bin_name,
			app_name,
			output_final_path,
			platform,
		} => match platform {
			Package::Macos {
				link_into_applications,
				link_for_bundle,
				open,
			} => {
				let sdk_root = get_sdk_root();
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
					[("SDKROOT", sdk_root.as_str())],
				);
				const SILICON_TRIPLE: &str = "aarch64-apple-darwin";
				let silicon_build =
					Utf8PathBuf::from(format!("target/{SILICON_TRIPLE}/release/{bin_name}"));
				assert!(silicon_build.is_file());

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
					[("SDKROOT", sdk_root.as_str())],
				);
				const INTEL_TRIPLE: &str = "x86_64-apple-darwin";
				let intel_build = Utf8PathBuf::from(format!("target/{INTEL_TRIPLE}/release/{bin_name}"));
				assert!(intel_build.is_file());

				let combined_bin_file =
					Utf8PathBuf::from(format!("target/release/{bin_name}", bin_name = bin_name));
				exec(
					"lipo",
					[
						"-create",
						"-output",
						combined_bin_file.as_str(),
						silicon_build.as_str(),
						intel_build.as_str(),
					],
				);

				// prepare package_path
				let release_dir_macos = Utf8PathBuf::from(format!("{}/macos", release_dir));
				let release_dir_src = format!("{}/src", release_dir_macos);
				let release_dir_package = format!("{}/{}.app", release_dir_src, app_name);
				if remove_dir_all(&release_dir_src).is_ok() {
					debug!("Removed old package src/ at {}", release_dir_src);
				}
				create_dir_all(&release_dir_package).expect("Unable to create package directory");

				// copy assets, binary and eventually credits
				let macos_contents_dir = Utf8PathBuf::from(format!("{}/Contents", &release_dir_package));
				{
					let assets_dir = Utf8PathBuf::from(format!("{}/MacOS/assets", &macos_contents_dir));
					create_dir_all(&assets_dir).unwrap();
					// copies assets
					exec("cp", ["-r", "assets/", assets_dir.as_str()]);
					let final_bin_file = Utf8PathBuf::from(format!(
						"{}/MacOS/{}",
						&macos_contents_dir,
						bin_name,
					));
					exec("cp", [combined_bin_file.as_str(), final_bin_file.as_str()]);
					exec("strip", [final_bin_file.as_str()]);
				}

				// copy over contents in build/macos
				let macos_build_dir = Utf8PathBuf::from(format!("{}/macos.app", build_dir));
				exec(
					"cp",
					[
						format!("{}/Contents/Info.plist", macos_build_dir).as_str(),
						format!("{}/Info.plist", macos_contents_dir).as_str(),
					],
				);
				create_dir(format!("{}/Resources", macos_contents_dir)).unwrap();
				exec(
					"cp",
					[
						format!("{}/Contents/Resources/AppIcon.icns", macos_build_dir).as_str(),
						format!("{}/Resources/AppIcon.icns", macos_contents_dir).as_str(),
					],
				);

				if link_for_bundle {
					// ln -s /Applications into the bundle
					exec("ln", ["-s", "/Applications", &release_dir_src]);
				}

				// put into volume
				let version = get_current_version();
				let dmg_name = format!("{} v{}.dmg", app_name, version);
				let final_dmg = Utf8PathBuf::from(format!("{}/{}", release_dir_macos, dmg_name));
				if final_dmg.is_file() {
					debug!("Removing old dmg: {}", final_dmg);
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
						release_dir_src.as_str(),
						final_dmg.as_str(),
					],
				);

				// if link, ln -s into /Applications
				if link_into_applications {
					let app_link = Utf8PathBuf::from(format!("/Applications/{}.app", app_name));
					if app_link.is_symlink() || app_link.is_file() {
						debug!("Removing old app link: rm -rf \"{}\"", app_link);
						remove_file(&app_link).unwrap();
					}
					exec("ln", ["-s", &release_dir_package, app_link.as_str()]);
				}

				if open {
					info!("Opening application ...");
					exec("open", [release_dir_package.as_str()]);
				}

				// eventually, code sign and notarize here

				info!("Successfully packaged macos application: {}", final_dmg);
				
				if output_final_path {
					println!("{}", final_dmg);
				}
			}
			Package::Windows => {
				const TARGET_TRIPLE: &str = "x86_64-pc-windows-gnu";
				cargo_exec([
					"build",
					"--release",
					"--target",
					TARGET_TRIPLE,
					"--no-default-features",
					"--features",
					"release",
				]);
				let bin_path = Utf8PathBuf::from(format!(
					"target/{TARGET_TRIPLE}/release/{bin_name}.exe",
					bin_name = bin_name
				));
				assert!(bin_path.is_file());

				let release_dir_windows = Utf8PathBuf::from(format!("{}/windows", release_dir));
				let release_dir_src = Utf8PathBuf::from(format!("{}/src", release_dir_windows));
				if remove_dir_all(PathBuf::from(&release_dir_src)).is_ok() {
					info!("Removed old release {}", release_dir_src);
				}
				create_dir_all(&release_dir_src).unwrap();

				// copy assets, binary and eventually credits
				exec(
					"cp",
					[
						"-r",
						"assets/",
						format!("{}/assets", release_dir_src).as_str(),
					],
				);
				let moved_exec = Utf8PathBuf::from(format!("{}/{}.exe", release_dir_src, bin_name));
				exec("cp", [bin_path.as_str(), moved_exec.as_str()]);
				assert!(moved_exec.is_file());

				// put into zip
				let version = get_current_version();
				let final_zip_name = format!("{} v{}.zip", app_name, version);
				let src_zip = Utf8PathBuf::from(format!("{}/{}", release_dir_src, final_zip_name));
				// cwd into target/windows/src/
				{
					let original_cwd = std::env::current_dir().unwrap();
					let original_cwd: &Utf8Path = Utf8Path::from_path(original_cwd.as_path()).unwrap();
					std::env::set_current_dir(&release_dir_src).unwrap();

					trace!(
						"Now in CWD {}, with the zip outputted at {}",
						release_dir_src,
						final_zip_name
					);

					exec("zip", ["-r", &final_zip_name, "."]);

					std::env::set_current_dir(original_cwd).unwrap();
					trace!("Back to normal CWD {}", original_cwd);
				}

				// mv from src/Space CRaft v0.0.0 to {release_folder}/Space carft v0.0.0
				exec("mv", [src_zip.as_str(), release_dir_windows.as_str()]);
				let final_zip = Utf8PathBuf::from(format!("{}/{}", release_dir_windows, final_zip_name));
				assert!(final_zip.is_file());

				info!("Successfully packaged windows application: {}", final_zip);

				if output_final_path {
					println!("{}", final_zip);
				}
			}
		},
	}

	// match args {
	// 	Cli::Package(Package {
	// 		platform,
	// 		bin_name,
	// 		app_name,
	// 		macos_link_into_applications: link_into_applications,
	// 		macos_link_for_bundle: link_for_bundle,
	// 		macos_open: open,
	// 	}) => match platform {
	// 		Platform::Windows => {

	// 		}
	// 		#[cfg(not(target_os = "macos"))]
	// 		Platform::MacOS => {
	// 			unimplemented!("Building for MacOS from a non-macos platform is not supported. Please run this command from a macos machine.")
	// 		}
	// 		#[cfg(target_os = "macos")]
	// 		Platform::MacOS => {
	// macos packaging

	// }
	// 	},
	// 	Cli::Prepare(Prepare {
	// 		platform,
	// 		// user_name,
	// 	}) => match platform {
	// 		Platform::Windows => {
	// 			// exec("rustup", ["target", "add", "x86_64-pc-windows-msvc"]);
	// 			exec("rustup", ["target", "add", "x86_64-pc-windows-gnu"]);
	// 			// cargo_exec(["install", "xwin"]);
	// 			// exec(
	// 			// 	"xwin",
	// 			// 	[
	// 			// 		"--accept-license",
	// 			// 		"splat",
	// 			// 		"--disable-symlinks",
	// 			// 		"--output",
	// 			// 		format!("/Users/{}/.xwin", user_name).as_str(),
	// 			// 	],
	// 			// );
	// 			#[cfg(target_os = "macos")]
	// 			exec("brew", ["install", "llvm"]);
	// 			#[cfg(target_os = "macos")]
	// 			exec("brew", ["install", "mingw-w64"]);
	// 		}
	// 		Platform::MacOS => {
	// 			exec("rustup", ["target", "add", "aarch64-apple-darwin"]);
	// 			exec("rustup", ["target", "add", "x86_64-apple-darwin"]);

	// 			// sort out icons
	// 			let build_dir = "build/macos.app/Contents";
	// 			let icon_dir = format!("{}/AppIcon.iconset", build_dir);
	// 			if remove_dir_all(&icon_dir).is_ok() {
	// 				println!("Removed old iconset dir");
	// 			}
	// 			create_dir(&icon_dir).unwrap();

	// 			let base_icon = "assets/images/icon_1024x1024.png";
	// 			assert!(Path::new(base_icon).exists());

	// 			let sips = |size: u16| {
	// 				exec(
	// 					"sips",
	// 					[
	// 						"-z",
	// 						size.to_string().as_str(),
	// 						size.to_string().as_str(),
	// 						base_icon,
	// 						"--out",
	// 						format!("{}/icon_{}x{}.png", icon_dir, size, size).as_str(),
	// 					],
	// 				);
	// 			};
	// 			let sips2 = |size: u16| {
	// 				assert!(size > 16);
	// 				exec(
	// 					"sips",
	// 					[
	// 						"-z",
	// 						size.to_string().as_str(),
	// 						size.to_string().as_str(),
	// 						base_icon,
	// 						"--out",
	// 						format!("{}/icon_{}x{}@2x.png", icon_dir, size / 2, size / 2).as_str(),
	// 					],
	// 				);
	// 			};
	// 			for size in [16, 32, 128, 256, 512].iter() {
	// 				sips(*size);
	// 			}
	// 			for size in [32, 64, 256, 512].iter() {
	// 				sips2(*size);
	// 			}

	// 			exec(
	// 				"iconutil",
	// 				[
	// 					"-c",
	// 					"icns",
	// 					&icon_dir,
	// 					"--output",
	// 					format!("{}/Resources/AppIcon.icns", build_dir).as_str(),
	// 				],
	// 			);
	// 		}
	// 	},
	// 	Cli::Update => {
	// 		cargo_exec(["update"]);
	// 		exec("rustup", ["update"]);
	// 		#[cfg(target_os = "macos")]
	// 		exec("brew", ["update"]);
	// 		#[cfg(target_os = "macos")]
	// 		exec("brew", ["upgrade"]);
	// 	}
	// 	Cli::Release(Release {
	// 		all,
	// 		windows,
	// 		macos,
	// 		title,
	// 		version,
	// 		proper_release,
	// 		dev_patch,
	// 	}) => {
	// 		if !all && !windows && !macos {
	// 			error!("You must specify at least one platform to release, e.g. --macos or --all");
	// 			std::process::exit(1);
	// 		}

	// 		let current_vers = get_current_version();
	// 		let current_version = current_vers.parse::<semver::Version>().expect("Current version is not valid");

	// 		let finalized_new_version;

	// 		match (version, dev_patch) {
	// 			(None, false) => {
	// 				error!("Must provide either --version 0.1.2-dev.3 or --dev-patch");
	// 				std::process::exit(1);
	// 			}
	// 			(Some(_), true) => {
	// 				error!("Cannot provide both --version and --dev-patch");
	// 				std::process::exit(1);
	// 			}
	// 			(Some(ver), false) => {
	// 				let new_version = ver.parse::<semver::Version>().expect("New version is not valid");
	// 				if new_version <= current_version {
	// 					error!("Version {} is already the current version or less, please provide a version greater than {}", new_version, current_vers);
	// 					std::process::exit(1);
	// 				} else {
	// 					finalized_new_version = format!("{}", new_version);
	// 				}
	// 			}
	// 			(None, true) => {
	// 				trace!("Incrementing dev patch version from {}", current_version);
	// 				assert!(current_version.build.is_empty());
	// 				if current_version.pre.is_empty() {
	// 					trace!("Prerelease is empty, adding -dev.1");
	// 					finalized_new_version = format!("{}-dev.1", current_version);
	// 				} else {
	// 					// extract last number after decimal place, increment by one
	// 					let pre = current_version.pre.as_str();
	// 					let num = pre.split('.').last().unwrap().parse::<u64>().unwrap();
	// 					let mut pre = current_version.clone();
	// 					pre.pre = Prerelease::EMPTY;
	// 					finalized_new_version = format!("{}-dev.{}", pre, num + 1);
	// 				}
	// 			}
	// 		}
	// 		assert!(finalized_new_version.parse::<semver::Version>().is_ok());
	// 		debug!("Finalized version for release: {}", finalized_new_version);

	// 		set_current_version(&finalized_new_version);

	// 		let release_dir = format!("release/gh-releases/{}", finalized_new_version);

	// 		if windows || all {
	// 			xtask_exec(["package", "windows"]);
	// 		}
	// 		if macos || all {
	// 			xtask_exec(["package", "macos"]);
	// 		}

	// 	}
	// }
}
