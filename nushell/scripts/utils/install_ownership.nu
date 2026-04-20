#!/usr/bin/env nu

use install_ownership_report.nu evaluate_install_ownership_report

export def has_home_manager_managed_install [] {
    (evaluate_install_ownership_report).has_home_manager_managed_install
}
