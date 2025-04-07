/*
Copyright (C) 2025 MaskedRedstonerProZ

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

// SPDX-License-Identifier: GPL-3.0-or-later

use std::env;
use std::fs;
use std::io::Write;
use std::process;
use std::process::{Command, Stdio};

/**
The name of the folder which houses the password store structure
*/
const FOLDER_NAME: &str = ".password-store";

fn main() {
    verify_installed("rofi");

    verify_installed("pass");

    let home_dir_envvar: String = String::from(env::var("HOME").unwrap());

    let password_path = format!("{}/{}", home_dir_envvar, FOLDER_NAME);

    if !fs::exists(password_path.clone().as_str()).unwrap() {
        println!("The pass utility must be initialized!");
        process::exit(1);
    }

    if fs::read_dir(password_path.clone()).unwrap().count() == 0 {
        println!("The password store must not be empty!");
        process::exit(1);
    }

    let full_paths = list_file_paths(password_path.as_str());

    let paths: Vec<String> = full_paths
        .iter()
        .map(|path| {
            path.strip_prefix(format!("{}/", password_path).as_str())
                .unwrap()
                .to_string()
        })
        .map(|path| path.strip_suffix(".gpg").unwrap().to_string())
        .collect();

    // println!("{:?}", paths);

    // Call rofi and get the selected item
    match rofi_select(&paths) {
        Some(selected) => {
            let mut pass = Command::new("pass")
                .args(&["-c"])
                .args(&[selected])
                .stdout(Stdio::inherit())
                .spawn()
                .expect("Failed to run pass");

            let _ = pass.wait().unwrap().success();
        }
        None => println!("No selection made."),
    }
}

fn rofi_select(options: &[String]) -> Option<String> {
    // Join the options with newlines
    let input = options.join("\n");

    let mut rofi = Command::new("rofi")
        .args(&["-dmenu"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("Failed to spawn rofi");

    // Write the options into rofi's stdin
    if let Some(stdin) = rofi.stdin.as_mut() {
        stdin
            .write_all(input.as_bytes())
            .expect("Failed to write to stdin");
    }

    // Capture the output
    let output = rofi.wait_with_output().expect("Failed to read stdout");
    if output.status.success() {
        // Trim any newline or whitespace from the result
        let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if selected.is_empty() {
            None
        } else {
            Some(selected)
        }
    } else {
        None
    }
}

fn list_file_paths(dir_path: &str) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();

    fs::read_dir(dir_path).unwrap().for_each(|path_result| {
        let path = path_result.unwrap();

        if !path.file_name().to_str().unwrap().starts_with(".") {
            if path.path().is_dir() {
                paths.append(&mut list_file_paths(path.path().to_str().unwrap()))
            }

            if path.path().is_file() {
                paths.push(path.path().to_str().unwrap().to_string());
            }
        }
    });

    paths
}

fn verify_installed(program: &str) {
    let path_envvar: String = String::from(env::var("PATH").unwrap());

    let mut flag = false;

    if path_envvar.split(":").any(|path| {
        if fs::exists(path).unwrap() {
            fs::read_dir(path)
                .unwrap()
                .all(|path| path.unwrap().path().to_str().unwrap() == program)
        } else {
            false
        }
    }) {
        flag = true;
    }

    if !flag {
        println!("The program {} must be installed on your system", program);
        process::exit(1);
    }
}

// fn log(value: &str) -> &str {
//     println!("{}", value);
//     value
// }
