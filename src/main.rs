use cargo::core::compiler::{CompileKind, RustcTargetData};
use cargo::core::resolver::{CliFeatures, ForceAllTargets, HasDevUnits};
use cargo::core::{FeatureValue, PackageIdSpec, Workspace};
use cargo::util::command_prelude::{App, Arg};
use cargo::util::interning::InternedString;
use cargo::{ops, Config};
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::path::PathBuf;
use std::rc::Rc;
use std::{env, io};
use thiserror::Error;
use RealDepsError::*;

type Result<T, E = RealDepsError> = std::result::Result<T, E>;

#[derive(Debug, Error)]
enum RealDepsError {
    #[error("cargo error: {0}")]
    Cargo(anyhow::Error),
    #[error("bad path: {0}")]
    BadPath(io::Error),
    #[error("Cargo.toml not found in {0}")]
    NoCargoToml(PathBuf),
}

fn main() -> Result<()> {
    let app = App::new(env!("CARGO_PKG_NAME"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .bin_name("cargo real-deps")
        .arg(
            Arg::with_name("all-features")
                .long("all-features")
                .help("Activate all features"),
        )
        .arg(
            Arg::with_name("no-default-features")
                .long("no-default-features")
                .help("Deactivate default features"),
        )
        .arg(
            Arg::with_name("features")
                .long("features")
                .takes_value(true)
                .help("Activates some features")
                .conflicts_with("all-features"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("Prints only the total number of dependencies"),
        )
        .arg(
            Arg::with_name("path")
                .takes_value(true)
                .help("Project directory, or path to Cargo.toml"),
        );

    // Hacky solution to ignore unexpected arg when being run as a cargo subcommand
    // If called as a subcommand, env::args is [executable-path, subcommand name, args, ...]
    // The subcommand name being present messes up the parsing so we filter it out
    let matches = app.get_matches_from(
        env::args_os().filter(|s| !s.eq_ignore_ascii_case("real-deps")),
    );

    let path = get_cargo_toml_path(matches.value_of_os("path"))?;

    let features = Rc::new(
        matches
            .values_of("features")
            .map(|v| {
                v.map(InternedString::new)
                    .map(FeatureValue::new)
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default(),
    );
    let cli_features = CliFeatures {
        features,
        all_features: matches.is_present("all-features"),
        uses_default_features: !matches.is_present("no-default-features"),
    };

    let config = Config::default().map_err(Cargo)?;
    let ws = Workspace::new(&path, &config).map_err(Cargo)?;
    let specs = vec![PackageIdSpec::from_package_id(
        ws.current().unwrap().package_id(),
    )];

    let targets = &[CompileKind::Host][..];
    let resolve = ops::resolve_ws_with_opts(
        &ws,
        &RustcTargetData::new(&ws, targets).map_err(Cargo)?,
        targets,
        &cli_features,
        &specs,
        HasDevUnits::No,
        ForceAllTargets::No,
    )
    .map_err(Cargo)?
    .targeted_resolve;

    let package_ids = resolve.sort();

    if matches.is_present("count") {
        eprint!("Total dependencies: ");
        println!("{}", package_ids.len());
    } else {
        package_ids.iter().for_each(|id| {
            eprintln!(
                "{} {} {:?}",
                id.name(),
                id.version(),
                resolve.features(*id)
            )
        });
    }

    Ok(())
}

fn get_cargo_toml_path(flag_val: Option<&OsStr>) -> Result<PathBuf> {
    let user_path = flag_val
        .map(PathBuf::from)
        .map(Result::Ok)
        .unwrap_or_else(|| env::current_dir().map_err(BadPath))?;
    let mut path = user_path.canonicalize().map_err(BadPath)?;
    if !path.ends_with("Cargo.toml") {
        if !path.is_dir() {
            return Err(BadPath(io::Error::new(
                io::ErrorKind::Unsupported,
                "not a Cargo.toml",
            )));
        }
        path.push("Cargo.toml");
    }

    if path.exists() {
        Ok(path)
    } else {
        // Returning user_path here means that the path shown in the error to
        // the user should more closely resemble what they're expecting, as it
        // won't be modified at all from their input
        Err(NoCargoToml(user_path))
    }
}
