use std::path::PathBuf;

use anyhow::Result;
use charon::{Global, GlobalRegistry, PackageTitle, Registry, SourcePackage};
use clap::{Parser, Subcommand};

#[derive(Parser, Debug, Clone)]
#[command(version, about="CLI to the Charon Packaging System", long_about=None)]
struct MainArgs {
    #[arg(short = 'r', long = "registry", help = "Root path to package registry")]
    registry_path: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug, Clone)]
enum Commands {
    NewPackage(NewPackageArgs),
    RemovePackage(RemovePackageArgs),
}

#[derive(Parser, Debug, Clone)]
#[command(version, about="Create a new Package Registry", long_about=None)]
struct NewPackageArgs {
    name: String,
    initial_version: String,
}

#[derive(Parser, Debug, Clone)]
#[command(version, about="Remove a package completely from the registry", long_about=None)]
struct RemovePackageArgs {
    name: String,
}

fn main() -> Result<()> {
    let args = MainArgs::parse();
    let cwd = std::env::current_dir()?;
    match args.command {
        Commands::NewPackage(new_args) => {
            let r = Registry::new(args.registry_path.clone().unwrap_or(cwd.clone()));
            let sp = SourcePackage {
                title: PackageTitle {
                    name: new_args.name.clone(),
                    version: new_args.initial_version,
                },
                description: "Please modify this description".into(),
                ..Default::default()
            };
            r.write(&sp)?;
            let gr = GlobalRegistry::new(args.registry_path.unwrap_or(cwd));
            let g = Global {
                name: new_args.name,
                ..Default::default()
            };
            gr.set(&g)?;
        }
        Commands::RemovePackage(rp_args) => {
            let r = Registry::new(args.registry_path.clone().unwrap_or(cwd.clone()));
            let gr = GlobalRegistry::new(args.registry_path.unwrap_or(cwd));
            r.remove(&rp_args.name)?;
            gr.remove(&rp_args.name)?;
        }
    }

    Ok(())
}
