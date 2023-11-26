use clap::{Parser, Subcommand};

#[derive(Parser)] // requires `derive` feature
#[command(bin_name = "cargo xtask")]
#[command(author, version, about, long_about = None)]
enum Cli {
	Release(Release),
	Dev(Dev),
	Setup(Setup),
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

	#[arg(long, short)]
	user_name: String,
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

fn main() {
	let args = Cli::parse();

	match args {
		Cli::Release(Release { platform }) => match platform {
			Platform::Windows => {
				cargo_exec(["build", "--release", "--target", "x86_64-pc-windows-gnu"]);
			}
			_ => todo!(),
		},
		Cli::Setup(Setup {
			platform,
			user_name,
		}) => match platform {
			Platform::Windows => {
				exec("rustup", ["target", "add", "x86_64-pc-windows-msvc"]);
				cargo_exec(["install", "xwin"]);
				exec(
					"xwin",
					[
						"--accept-license",
						"splat",
						"--disable-symlinks",
						"--output",
						format!("/Users/{}/.xwin", user_name).as_str(),
					],
				);
				exec("brew", ["install", "llvm"]);
				exec("brew", ["install", "mingw-w64"]);
			}
			_ => todo!(),
		},
		_ => todo!(),
	}
}
