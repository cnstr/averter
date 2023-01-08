#![warn(clippy::all)]
#![warn(clippy::correctness)]
#![warn(clippy::suspicious)]
#![warn(clippy::style)]
#![warn(clippy::complexity)]
#![warn(clippy::perf)]

use serde_json::Value;
use serde_yaml::{from_str as from_yaml_str, to_string as to_yaml_string};
use std::{fs::write, path::Path};

fn main() {
	let cargo_version = env!("CARGO_PKG_VERSION");
	let image_tag = format!("tale.me/canister/averter:{cargo_version}");

	let path = Path::new("./kubernetes/averter.yaml");
	let raw_manifest = match std::fs::read_to_string(path) {
		Ok(manifest) => manifest,
		Err(err) => {
			panic!("Failed to read averter.yaml ({})", err)
		}
	};
	let kube_objects = raw_manifest.split("---").collect::<Vec<&str>>();
	let mut manifest: Value = match from_yaml_str(kube_objects[1]) {
		Ok(manifest) => manifest,
		Err(err) => {
			panic!("Failed to parse averter.yaml ({})", err)
		}
	};

	manifest["spec"]["template"]["spec"]["containers"][0]["image"] = Value::String(image_tag);
	let stringified_manifest = match to_yaml_string(&manifest) {
		Ok(manifest) => manifest,
		Err(err) => {
			panic!("Failed to stringify averter.yaml ({})", err)
		}
	};

	match write(
		path,
		format!("{}---\n{}", kube_objects[0], stringified_manifest),
	) {
		Ok(_) => {}
		Err(err) => {
			panic!("Failed to write averter.yaml ({})", err)
		}
	}
}
