use std::path::PathBuf;
use yazelix_core::repo_issue_sync::run_issue_sync;
use yazelix_core::repo_contract_validation::sync_readme_surface;
use yazelix_core::repo_version_bump::perform_version_bump;
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
        "version-bump" => {
            let target_version = parse_version_bump_args(args.collect());
            perform_version_bump(&resolved_repo_root, &target_version).map(|result| {
                println!(
                    "{}",
                    serde_json::json!({
                        "previous_version": result.previous_version,
                        "target_version": result.target_version,
                        "release_date": result.release_date,
                        "commit_message": result.commit_message,
                        "commit_sha": result.commit_sha,
                        "tag": result.tag,
                    })
                );
            })
        }
        "sync-issues" => {
            let dry_run = parse_sync_issues_args(args.collect());
            run_issue_sync(&resolved_repo_root, dry_run).map(|summary| {
                println!(
                    "{}",
                    serde_json::json!({
                        "created": summary.created,
                        "reopened": summary.reopened,
                        "closed": summary.closed,
                        "unchanged": summary.unchanged,
                        "comments_created": summary.comments_created,
                        "comments_updated": summary.comments_updated,
                        "comments_unchanged": summary.comments_unchanged,
                    })
                );
            })
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
        "Usage: yzx_repo_maintainer [--repo-root PATH] <sync-readme-surface|run-tests|version-bump|sync-issues> [options]"
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

fn parse_version_bump_args(args: Vec<String>) -> String {
    let mut iter = args.into_iter();
    let Some(version) = iter.next() else {
        eprintln!("Missing VERSION for version-bump");
        std::process::exit(2);
    };
    if let Some(extra) = iter.next() {
        eprintln!("Unexpected version-bump argument `{extra}`");
        std::process::exit(2);
    }
    version
}

fn parse_sync_issues_args(args: Vec<String>) -> bool {
    let mut dry_run = false;
    for arg in args {
        match arg.as_str() {
            "--dry-run" => dry_run = true,
            _ => {
                eprintln!("Unknown sync-issues option `{arg}`");
                std::process::exit(2);
            }
        }
    }
    dry_run
}
