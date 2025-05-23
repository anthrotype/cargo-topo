use clap::{Parser, Subcommand};
use clap_cargo::style::CLAP_STYLING;
use guppy::MetadataCommand;
use guppy::graph::{DependencyDirection, PackageGraph, PackageSet};
use std::collections::HashSet;

#[derive(Parser)]
#[command(name = "cargo")]
#[command(bin_name = "cargo")]
#[command(styles = CLAP_STYLING)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List workspace crates in topological dependency order
    Topo(TopoArgs),
}

#[derive(Parser)]
struct TopoArgs {
    /// Path to workspace root (defaults to current directory)
    #[arg(short, long)]
    manifest_path: Option<std::path::PathBuf>,

    /// Show dependencies in reverse order (dependents first)
    #[arg(short, long)]
    reverse: bool,

    /// Include dev-dependencies in analysis
    #[arg(short, long)]
    include_dev: bool,

    /// Show all dependencies (including external dependencies).
    /// By default, only show workspace members.
    #[arg(short, long, default_value = "false")]
    all: bool,

    /// Output compact line-separated list of crate names only
    #[arg(short, long)]
    compact: bool,

    /// Select a specific package as the root of the dependency tree
    #[arg(short = 'p', long = "package")]
    package: Option<String>,
    
    /// Exclude specific workspace members from the output
    #[arg(long)]
    exclude: Vec<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Topo(args) => run_topo_command(args),
    }
}

fn run_topo_command(args: TopoArgs) -> Result<(), Box<dyn std::error::Error>> {
    // Get cargo metadata
    let mut metadata_cmd = MetadataCommand::new();
    if let Some(manifest_path) = &args.manifest_path {
        metadata_cmd.manifest_path(manifest_path);
    }

    // Build the package graph
    let package_graph = metadata_cmd.build_graph()?;

    // Get workspace members and filter out excluded packages
    let workspace_members = package_graph.resolve_workspace();
    let excluded: HashSet<&str> = args.exclude.iter().map(String::as_str).collect();
    let workspace_ids: HashSet<_> = workspace_members
        .package_ids(DependencyDirection::Forward)
        .filter(|pkg_id| {
            let pkg = package_graph.metadata(pkg_id).unwrap();
            !excluded.contains(pkg.name())
        })
        .collect();

    // Determine the dependency set based on root package selection
    let dependency_set = if let Some(root_package) = &args.package {
        // Find the root package by name
        let root_pkg = package_graph
            .packages()
            .find(|pkg| pkg.name() == root_package)
            .ok_or_else(|| format!("Package '{}' not found in workspace", root_package))?;

        // Query dependencies starting from root package, filtering dev-only deps
        package_graph.query_directed(
            std::iter::once(root_pkg.id()), 
            DependencyDirection::Forward
        )?.resolve_with_fn(|_query, link| {
            // Include the link unless it's dev-only and we're not including dev deps
            !link.dev_only() || args.include_dev
        })
    } else {
        // Query full workspace
        package_graph.query_workspace().resolve()
    };

    if args.compact {
        show_compact_output(&dependency_set, &workspace_ids, args.reverse, args.all)?;
    } else {
        if args.package.is_some() {
            let root_name = args.package.as_ref().unwrap();
            if args.reverse {
                println!("Dependencies from '{}' in reverse topological order:", root_name);
            } else {
                println!("Dependencies from '{}' in topological order:", root_name);
            }
        } else {
            if args.reverse {
                println!("Workspace crates in reverse topological order:");
            } else {
                println!("Workspace crates in topological order:");
            }
        }
        if !args.exclude.is_empty() {
            println!("Excluding: {}", args.exclude.join(", "));
        }
        println!();
        
        if args.all {
            show_all_dependencies_topological_order(&dependency_set, &workspace_ids, args.reverse)?;
        } else {
            show_workspace_topological_order(&dependency_set, &workspace_ids, args.reverse)?;
        }
        
        if args.include_dev {
            println!("\nDev-dependencies analysis:");
            show_dev_dependencies(&package_graph, &dependency_set, &workspace_ids)?;
        }
    }

    Ok(())
}

fn show_workspace_topological_order(
    dependency_set: &PackageSet,
    workspace_ids: &HashSet<&guppy::PackageId>,
    reverse: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // in guppy DependencyDirection, the logic is reversed from traditional topological order
    let direction = if reverse {
        DependencyDirection::Forward // dependent packages appear before their dependencies
    } else {
        DependencyDirection::Reverse // dependencies appear first by default
    };
    // Iterate in topological order, filtering for workspace members only
    for package in dependency_set.packages(direction) {
        if workspace_ids.contains(&package.id()) {
            let metadata = package;
            println!("📦 {} ({})", metadata.name(), metadata.version());

            // Show direct dependencies within workspace
            let deps: Vec<_> = package
                .direct_links()
                .filter(|link| !link.dev_only()) // Exclude dev dependencies
                .filter(|link| workspace_ids.contains(&link.to().id()))
                .map(|link| link.to().name())
                .collect();

            if !deps.is_empty() {
                println!("   └─ depends on: {}", deps.join(", "));
            }
            println!();
        }
    }

    Ok(())
}

fn show_all_dependencies_topological_order(
    dependency_set: &PackageSet,
    workspace_ids: &HashSet<&guppy::PackageId>,
    reverse: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // in guppy DependencyDirection, the logic is reversed from traditional topological order
    let direction = if reverse {
        DependencyDirection::Forward // dependent packages appear before their dependencies
    } else {
        DependencyDirection::Reverse // dependencies appear first by default
    };

    for package in dependency_set.packages(direction) {
        let is_workspace = workspace_ids.contains(&package.id());
        let prefix = if is_workspace { "📦" } else { "📄" };

        println!("{} {} ({})", prefix, package.name(), package.version());

        if is_workspace {
            // For workspace members, show their direct dependencies
            let deps: Vec<_> = package
                .direct_links()
                .filter(|link| !link.dev_only())
                .map(|link| {
                    let to_package = link.to();
                    let is_workspace_dep = workspace_ids.contains(&to_package.id());
                    let marker = if is_workspace_dep { "📦" } else { "📄" };
                    format!("{} {}", marker, to_package.name())
                })
                .collect();

            if !deps.is_empty() {
                println!("   └─ depends on: {}", deps.join(", "));
            }
        }
        println!();
    }

    Ok(())
}

fn show_dev_dependencies(
    package_graph: &PackageGraph,
    dependency_set: &PackageSet,
    workspace_ids: &HashSet<&guppy::PackageId>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Only analyze dev-dependencies for packages that are in our filtered dependency set
    let packages_in_scope: HashSet<_> = dependency_set
        .package_ids(DependencyDirection::Forward)
        .collect();

    for package_id in workspace_ids {
        // Only show dev-dependencies for packages that are in our analysis scope
        if !packages_in_scope.contains(package_id) {
            continue;
        }

        let package = package_graph.metadata(package_id)?;

        let dev_deps: Vec<_> = package
            .direct_links()
            .filter(|link| link.dev_only())
            .map(|link| link.to().name())
            .collect();

        if !dev_deps.is_empty() {
            println!(
                "🧪 {} dev-dependencies: {}",
                package.name(),
                dev_deps.join(", ")
            );
        }
    }

    Ok(())
}

fn show_compact_output(
    dependency_set: &PackageSet,
    workspace_ids: &HashSet<&guppy::PackageId>,
    reverse: bool,
    all: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // in guppy DependencyDirection, the logic is reversed from traditional topological order
    let direction = if reverse {
        DependencyDirection::Forward // dependent packages appear before their dependencies
    } else {
        DependencyDirection::Reverse // dependencies appear first by default
    };

    let mut crate_names = Vec::new();

    for package in dependency_set.packages(direction) {
        let is_workspace = workspace_ids.contains(&package.id());

        if !all && !is_workspace {
            continue;
        }

        crate_names.push(package.name().to_string());
    }

    // Output as line-separated list
    println!("{}", crate_names.join("\n"));

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_current_workspace() {
        // Test with current workspace
        let metadata = guppy::MetadataCommand::new();
        let package_graph = metadata.build_graph().unwrap();
        let workspace_members = package_graph.resolve_workspace();

        // Should have at least one workspace member
        assert!(workspace_members.len() > 0);

        // Test topological ordering doesn't panic
        let workspace_set = package_graph.query_workspace().resolve();
        let packages: Vec<_> = workspace_set
            .packages(DependencyDirection::Forward)
            .collect();

        assert!(!packages.is_empty());
    }

    #[test]
    fn test_workspace_member_identification() {
        let metadata = guppy::MetadataCommand::new();
        let package_graph = metadata.build_graph().unwrap();
        let workspace_members = package_graph.resolve_workspace();
        let workspace_ids: HashSet<_> = workspace_members
            .package_ids(DependencyDirection::Forward)
            .collect();

        // Should be able to identify workspace members
        assert!(!workspace_ids.is_empty());

        // Each workspace member should be found in the package graph
        for id in workspace_ids {
            assert!(package_graph.metadata(id).is_ok());
        }
    }
}
