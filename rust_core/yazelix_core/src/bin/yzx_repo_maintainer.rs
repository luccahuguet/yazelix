use std::path::PathBuf;
use yazelix_core::repo_contract_validation::sync_readme_surface;
use yazelix_core::repo_test_runner::{RepoTestOptions, run_repo_tests};
use yazelix_core::repo_validation::repo_root;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut resolved_repo_root = repo_root();
    let Some(first_arg) = args.next() else {
        print_usage_and_exit();
    };

    let command = if first_arg == "--repo-root" {
        let Some(path) = args.next() else {
            eprintln!("Missing PATH after --repo-root");
            std::process::exit(2);
        };
        resolved_repo_root = PathBuf::from(path);
        args.next().unwrap_or_else(|| {
            print_usage_and_exit();
        })
    } else {
        first_arg
    };

    let result = match command.as_str() {
        "sync-readme-surface" => {
            let mut readme_path = None;
            let mut version = None;
            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--readme-path" => {
                        let Some(path) = args.next() else {
                            eprintln!("Missing value after --readme-path");
                            std::process::exit(2);
                        };
                        readme_path = Some(PathBuf::from(path));
                    }
                    "--version" => {
                        let Some(value) = args.next() else {
                            eprintln!("Missing value after --version");
                            std::process::exit(2);
                        };
                        version = Some(value);
                    }
                    _ => {
                        eprintln!("Unknown sync-readme-surface option `{arg}`");
                        std::process::exit(2);
                    }
                }
            }

            sync_readme_surface(
                &resolved_repo_root,
                readme_path.as_deref(),
                version.as_deref(),
            )
            .map(|sync| {
                println!(
                    "{}",
                    serde_json::json!({
                        "readme_path": sync.readme_path.display().to_string(),
                        "title_changed": sync.title_changed,
                        "series_changed": sync.series_changed,
                    })
                );
            })
        }
        "run-tests" => {
            let options = parse_run_tests_options(args.collect());
            run_repo_tests(&resolved_repo_root, &options)
        }
        _ => {
            eprintln!("Unknown maintainer command `{command}`");
            print_usage_and_exit();
        }
    };

    if let Err(error) = result {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn print_usage_and_exit() -> ! {
    eprintln!(
        "Usage: yzx_repo_maintainer [--repo-root PATH] <sync-readme-surface|run-tests> [options]"
    );
    std::process::exit(2);
}

fn parse_run_tests_options(args: Vec<String>) -> RepoTestOptions {
    let mut options = RepoTestOptions::default();
    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--verbose" | "-v" => options.verbose = true,
            "--new-window" | "-n" => options.new_window = true,
            "--lint-only" => options.lint_only = true,
            "--profile" => options.profile = true,
            "--sweep" => options.sweep = true,
            "--visual" => options.visual = true,
            "--all" | "-a" => options.all = true,
            "--delay" => {
                let Some(raw) = iter.next() else {
                    eprintln!("Missing value after --delay");
                    std::process::exit(2);
                };
                options.delay = raw.parse::<u64>().unwrap_or_else(|_| {
                    eprintln!("Invalid --delay value `{raw}`");
                    std::process::exit(2);
                });
            }
            _ => {
                eprintln!("Unknown run-tests option `{arg}`");
                std::process::exit(2);
            }
        }
    }
    options
}
