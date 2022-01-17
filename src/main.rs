use cargo::{
    core::{
        compiler::{CompileKind, RustcTargetData},
        resolver::{CliFeatures, ForceAllTargets, HasDevUnits},
        FeatureValue, PackageIdSpec, Workspace,
    },
    ops,
    util::{
        command_prelude::{App, Arg},
        config::Config,
        errors::CargoResult,
        interning::InternedString,
    },
};
use std::{collections::BTreeSet, env, path::PathBuf, rc::Rc};

fn main() -> CargoResult<()> {
    let app = App::new("cargo-real-deps")
        .arg(
            Arg::with_name("all-features")
                .long("all-features")
                .help("activate all features"),
        )
        .arg(
            Arg::with_name("no-default-features")
                .long("no-default-features")
                .help("deactivate default features"),
        )
        .arg(
            Arg::with_name("features")
                .long("features")
                .takes_value(true)
                .help("activates some features"),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .help("prints only the total number of dependencies"),
        )
        .arg(
            Arg::with_name("path")
                .takes_value(true)
                .help("path to (or directory containing) Cargo.toml"),
        );

    // Hacky solution to ignore unexpected arg when being run as a cargo subcommand
    // If called as a subcommand, env::args is [executable-path, subcommand name, args, ...]
    // The subcommand name being present messes up the parsing so we filter it out
    let matches =
        app.get_matches_from(env::args_os().filter(|s| !s.eq_ignore_ascii_case("real-deps")));

    let mut path: PathBuf = matches
        .value_of_os("path")
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("can't access current directory"))
        .canonicalize()
        .expect("failed to canonicalize path to Cargo.toml");

    if !path.ends_with("Cargo.toml") {
        assert!(path.is_dir(), "invalid file provided");
        path.push("Cargo.toml");
    }

    let all_features = matches.is_present("all-features");
    let uses_default_features = !matches.is_present("no-default-features");
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

    let config = Config::default()?;
    let ws = Workspace::new(&path, &config)?;
    let specs = vec![PackageIdSpec::from_package_id(
        ws.current().unwrap().package_id(),
    )];

    let targets = &[CompileKind::Host][..];
    let resolve = ops::resolve_ws_with_opts(
        &ws,
        &RustcTargetData::new(&ws, targets)?,
        targets,
        &CliFeatures {
            features,
            all_features,
            uses_default_features,
        },
        &specs,
        HasDevUnits::No,
        ForceAllTargets::No,
    )?
    .targeted_resolve;

    let package_ids = resolve.sort();

    if matches.is_present("count") {
        eprint!("Total dependencies: ");
        println!("{}", package_ids.len());
    } else {
        package_ids
            .iter()
            .for_each(|id| eprintln!("{} {} {:?}", id.name(), id.version(), resolve.features(*id)));
    }

    Ok(())
}
