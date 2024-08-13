// Toy Programming Language | by mealet
// https://github.com/mealet/tpl-lang
// =========================================
// Project licensed under the BSD-3 LICENSE.
// Check the `LICENSE` file to more info.

const PROJECT_NAME: &str = "tpl-lang";
const PROJECT_PACKAGE: &str = env!("CARGO_PKG_NAME");
const PROJECT_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROJECT_REPO: &str = "https://github.com/mealet/tpl-lang";

use colored::Colorize;

pub fn print_greeting() {
    let greeting = format!(
        "| {} - {}\n| {}",
        PROJECT_NAME, PROJECT_VERSION, PROJECT_REPO
    )
    .cyan();

    println!("{}", greeting);
}

pub fn print_usage() {
    let usage = format!(
        "| Usage: {}\n| Example: {}",
        format!("{} [input] [output]", PROJECT_PACKAGE).yellow(),
        format!("{} example.tpl output", PROJECT_PACKAGE).yellow()
    );

    println!("{}", usage);
}
