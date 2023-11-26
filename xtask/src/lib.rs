pub use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub fn get_cargo_path() -> String {
	// get cargo executable from env CARGO, and run it with str
	let cargo_exec_path = std::env::var("CARGO").unwrap();
	assert!(Path::new(&cargo_exec_path).is_file());
	cargo_exec_path
}

pub fn cargo_exec<'s>(args: impl IntoIterator<Item = &'s str>) {
	// get cargo executable from env CARGO, and run it with str
	let cargo_exec_path = get_cargo_path();
	exec(&cargo_exec_path, args);
}

pub fn exec<'a, 's>(exec_str: &'a str, args: impl IntoIterator<Item = &'s str>) -> String {
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

pub fn exec_with_envs<'a, 's>(
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

pub fn get_bin_name() -> String {
	// parse Cargo.toml for bin name
	let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
	let cargo_toml: toml::Value = toml::from_str(&cargo_toml).unwrap();
	let bin_name = cargo_toml["package"]["name"].as_str().unwrap();
	bin_name.to_string()
}

#[cfg(target_os = "macos")]
pub fn get_sdk_root() -> PathBuf {
	let str = exec("xcrun", ["-sdk", "macosx", "--show-sdk-path"]);
	let str = str.trim();
	let path = Path::new(str);

	assert!(path.exists(), "SDK path {str} does not exist");

	PathBuf::from(path)
}