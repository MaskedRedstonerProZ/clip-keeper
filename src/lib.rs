/*
 * Copyright (C) 2025 MaskedRedstonerProZ <maskedredstonerproz@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */

// SPDX-License-Identifier: GPL-3.0-or-later

use crate::AddPass::{ChooseDir, ChooseEntryType, ChooseFileName};
use crate::ChangePass::{ChooseFile, ChooseNewPassEntryType};
use rofi_mode::{Action, Api, Event, Matcher, Mode, export_mode};
use std::cmp::PartialEq;
use std::fmt::Debug;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, fs};


#[derive(Debug, PartialEq, Eq)]
enum Menu {
    Initial,
    CopyPass,
    AddPass(AddPass),
    ChangePass(ChangePass),
}

#[derive(Debug, PartialEq, Eq)]
enum AddPass {
    ChooseDir,
    ChooseFileName,
    ChooseEntryType,
}

#[derive(Debug, PartialEq, Eq)]
enum ChangePass {
    ChooseFile,
    ChooseNewPassEntryType,
}

#[derive(Debug, PartialEq, Eq)]
enum PasswordEntryType {
    UserInput,
}

pub struct ClipKeeperModi {
    entries: Vec<String>,
    previous_output: String,
    menu: Menu,
    passwd_dir: String,
    passwd_file_name: String,
    passwd_entry_type: PasswordEntryType,
}

const BACK_OPERATOR: &str = "..";

const INITIAL_MENU: [&str; 4] = [
    "Copy password",
    "Add new password",
    "Change password",
    "Quit",
];

const PASSWORD_ENTRY_TYPE_SELECTION_MENU: [&str; 2] = ["Input", "Generate"];

trait StrArrayExtensions {
    fn map_to_vec_string(&self) -> Vec<String>;
}

impl<const N: usize> StrArrayExtensions for [&str; N] {
    fn map_to_vec_string(&self) -> Vec<String> {
        self.iter().map(|entry| entry.to_string()).collect()
    }
}

impl ClipKeeperModi {
    fn autocomplete(&self, input: &str) -> rofi_mode::String {
        let entry_to_autocomplete = self
            .entries
            .iter()
            .find(|entry| entry.starts_with(input))
            .unwrap_or(&(self.entries[1]))
            .clone();

        let autocompletable_entry = entry_to_autocomplete
            .strip_prefix(&self.previous_output)
            .unwrap_or(entry_to_autocomplete.as_str());

        if autocompletable_entry.contains('/') {
            let parts: Vec<&str> = autocompletable_entry.split('/').collect();
            return rofi_mode::String::from(format!("{}{}", self.previous_output, parts[0]));
        }

        rofi_mode::String::from(format!("{}{}", self.previous_output, autocompletable_entry))
    }

    fn get_password_store_location(&self) -> PathBuf {
        if let Ok(dir) = env::var(String::from("PASSWORD_STORE_DIR")) {
            PathBuf::from(dir)
        } else {
            PathBuf::from(env::var(String::from("HOME")).unwrap_or_default())
                .join(String::from(".password-store"))
        }
    }

    fn list_dirs(&self, dir_path: String) -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();

        fs::read_dir(dir_path.clone())
            .unwrap()
            .for_each(|entry_result| {
                let entry = entry_result.unwrap();

                if !entry.file_name().to_str().unwrap().starts_with(".") {
                    if entry.path().is_dir() {
                        paths.push(
                            entry
                                .path()
                                .file_name()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap()
                                .to_string(),
                        );

                        let inner_entries =
                            self.list_dirs(entry.path().to_str().unwrap_or_default().to_string());

                        inner_entries.iter().for_each(|inner_entry| {
                            paths.push(format!(
                                "{}/{}",
                                entry
                                    .path()
                                    .file_name()
                                    .unwrap_or_default()
                                    .to_str()
                                    .unwrap(),
                                inner_entry
                            ));
                        });
                    }

                    if entry.path().is_file() {
                        paths.push(String::from("FILE_ENTRY"))
                    }
                }
            });

        paths
            .iter()
            .filter(|inner_entry| *(*inner_entry) != "FILE_ENTRY")
            .map(|inner_entry| (*inner_entry).clone())
            .collect()
    }

    fn strip_prefix(&self, items: Vec<String>) -> Vec<String> {
        let password_store_location = self
            .get_password_store_location()
            .to_str()
            .unwrap_or_default()
            .to_string();

        items
            .iter()
            .map(|item| {
                let segments: Vec<String> = item.split("/").map(|item| item.to_string()).collect();

                let mut final_item = String::new();

                segments.iter().for_each(|segment| {
                    if *segment == *segments.last().unwrap() {
                        let inner_segments: Vec<String> =
                            segment.split(".").map(|item| item.to_string()).collect();
                        final_item += inner_segments[0].as_str()
                    } else if !password_store_location.contains(segment) {
                        final_item += format!("{}/", segment).as_str()
                    }
                });

                final_item
            })
            .collect()
    }

    fn list_file_paths(&self, dir_path: String) -> Vec<String> {
        let mut paths: Vec<String> = Vec::new();

        fs::read_dir(dir_path).unwrap().for_each(|path_result| {
            let path = path_result.unwrap();

            if !path.file_name().to_str().unwrap().starts_with(".") {
                if path.path().is_dir() {
                    paths.append(
                        &mut self
                            .list_file_paths(path.path().to_str().unwrap_or_default().to_string()),
                    )
                }

                if path.path().is_file() {
                    paths.push(path.path().to_str().unwrap_or_default().to_string());
                }
            }
        });

        self.strip_prefix(paths)
    }

    fn run_pass(&self, command: &str, flags: &str, file_name: String) {
        let _ = Command::new(String::from("pass"))
            .args([command.to_string(), flags.to_string(), file_name])
            .stdin(Stdio::null())
            .stdout(Stdio::inherit())
            .spawn();
    }

    fn init_menu(
        &mut self,
        menu_content: &mut Vec<String>,
        new_menu: Menu,
        input: &mut rofi_mode::String,
    ) {
        let mut new_entries: Vec<String> = Vec::new();
        if !vec![
            Menu::Initial,
            Menu::AddPass(ChooseFileName),
            Menu::AddPass(ChooseEntryType),
            Menu::ChangePass(ChooseNewPassEntryType),
        ]
        .contains(&new_menu)
        {
            new_entries.push(BACK_OPERATOR.to_string());
        }
        new_entries.append(menu_content);
        self.entries = new_entries;
        self.menu = new_menu;
        self.previous_output = String::new();
        *input = rofi_mode::String::new();
    }
}

impl<'rofi> Mode<'rofi> for ClipKeeperModi {
    const NAME: &'static str = "clip-keeper\0";

    fn init(_api: Api<'rofi>) -> Result<Self, ()> {
        Ok(ClipKeeperModi {
            entries: INITIAL_MENU.map_to_vec_string(),
            previous_output: String::new(),
            menu: Menu::Initial,
            passwd_dir: String::new(),
            passwd_file_name: String::new(),
            passwd_entry_type: PasswordEntryType::UserInput,
        })
    }

    fn entries(&mut self) -> usize {
        self.entries.len()
    }

    fn entry_content(&self, line: usize) -> rofi_mode::String {
        rofi_mode::String::from(self.entries[line].clone())
    }

    fn react(&mut self, event: Event, input: &mut rofi_mode::String) -> Action {
        let password_store_location = self
            .get_password_store_location()
            .to_str()
            .unwrap_or_default()
            .to_string();

        if input.is_empty() {
            self.previous_output = String::new()
        }

        match event {
            Event::CustomCommand {
                number: 0,
                selected,
            } => {
                let selected_index = selected.unwrap_or_default();

                if self.menu != Menu::Initial {
                    if self.entries.iter().any(|entry| *entry == input.as_str()) {
                        return Action::SetMode(0);
                    }

                    let mut autocompletion_candidate = input.to_string();

                    if autocompletion_candidate.is_empty() {
                        if self.entries[selected_index] == BACK_OPERATOR {
                            autocompletion_candidate = self.entries[1].clone()
                        } else {
                            autocompletion_candidate = self.entries[selected_index].clone()
                        }
                    }

                    let output = self.autocomplete(autocompletion_candidate.as_str());
                    *input = output.clone();
                    self.previous_output =
                        format!("{}/", output.clone().parse::<String>().unwrap());
                    return Action::Reload;
                }

                Action::SetMode(0)
            }
            Event::Ok { selected, .. } => {
                if self.entries[selected] == INITIAL_MENU[0] {
                    let files = &mut self.list_file_paths(password_store_location);
                    self.init_menu(files, Menu::CopyPass, input);
                    return Action::Reload;
                }

                if self.entries[selected] == INITIAL_MENU[1] {
                    let dirs = &mut self.list_dirs(password_store_location);
                    self.init_menu(dirs, Menu::AddPass(ChooseDir), input);
                    return Action::Reload;
                }

                if self.entries[selected] == INITIAL_MENU[2] {
                    let files = &mut self.list_file_paths(password_store_location);
                    self.init_menu(files, Menu::ChangePass(ChooseFile), input);
                    return Action::Reload;
                }

                if self.entries[selected] == BACK_OPERATOR {
                    self.entries = INITIAL_MENU.map_to_vec_string();
                    self.menu = Menu::Initial;
                    *input = rofi_mode::String::new();
                    return Action::Reload;
                }

                if self.menu == Menu::CopyPass {
                    self.run_pass("show", "-c", self.entries[selected].clone());
                    return Action::Exit;
                }

                if self.menu == Menu::AddPass(ChooseDir) {
                    self.passwd_dir = self.entries[selected].clone();
                    self.init_menu(&mut Vec::new(), Menu::AddPass(ChooseFileName), input);
                    return Action::Reload;
                }

                if self.menu == Menu::AddPass(ChooseEntryType) {
                    if self.entries[selected] == PASSWORD_ENTRY_TYPE_SELECTION_MENU[0] {
                        self.passwd_entry_type = PasswordEntryType::UserInput;
                    } else if self.entries[selected] == PASSWORD_ENTRY_TYPE_SELECTION_MENU[1] {
                        self.run_pass(
                            "generate",
                            "-c",
                            format!("{}/{}", self.passwd_dir, self.passwd_file_name),
                        );
                        return Action::Exit;
                    }

                    println!(
                        "PASS_ADD: [PASSWD_DIR: {}, PASSWD_FILE_NAME: {}, PASSWD_ENTRY_TYPE: {:?}]",
                        self.passwd_dir, self.passwd_file_name, self.passwd_entry_type
                    );
                    return Action::Exit;
                }

                if self.menu == Menu::ChangePass(ChooseFile) {
                    self.passwd_file_name = self.entries[selected].clone();
                    self.init_menu(
                        &mut PASSWORD_ENTRY_TYPE_SELECTION_MENU.map_to_vec_string(),
                        Menu::ChangePass(ChooseNewPassEntryType),
                        input,
                    );
                    return Action::Reload;
                }

                if self.menu == Menu::ChangePass(ChooseNewPassEntryType) {
                    if self.entries[selected] == PASSWORD_ENTRY_TYPE_SELECTION_MENU[0] {
                        self.passwd_entry_type = PasswordEntryType::UserInput;
                    } else if self.entries[selected] == PASSWORD_ENTRY_TYPE_SELECTION_MENU[1] {
                        self.run_pass("generate", "-cf", self.passwd_file_name.clone());
                        return Action::Exit;
                    }

                    println!(
                        "PASS_CHNG: [PASSWD_FILE_NAME: {}, PASSWD_ENTRY_TYPE: {:?}]",
                        self.passwd_file_name, self.passwd_entry_type
                    );
                    return Action::Exit;
                }

                println!("{}", self.entries[selected]);
                Action::Exit
            }
            Event::CustomInput { .. } => {
                if self.menu == Menu::CopyPass {
                    self.run_pass("show", "-c", input.parse().unwrap());
                    return Action::Exit;
                }

                if self.menu == Menu::AddPass(ChooseDir) {
                    self.passwd_dir = input.to_string();
                    self.init_menu(&mut Vec::new(), Menu::AddPass(ChooseFileName), input);
                    return Action::Reload;
                }

                if self.menu == Menu::AddPass(ChooseFileName) {
                    self.passwd_file_name = input.to_string();
                    self.init_menu(
                        &mut PASSWORD_ENTRY_TYPE_SELECTION_MENU.map_to_vec_string(),
                        Menu::AddPass(ChooseEntryType),
                        input,
                    );
                    return Action::Reload;
                }

                println!("{}", input);
                Action::Exit
            }
            Event::Cancel { .. } => Action::Exit,
            _ => Action::SetMode(0),
        }
    }

    fn matches(&self, line: usize, matcher: Matcher<'_>) -> bool {
        matcher.matches(dbg!(&self.entries[line]))
    }

    fn message(&mut self) -> rofi_mode::String {
        match self.menu {
            Menu::CopyPass => rofi_mode::String::from(
                "Password file selection, choose the password you wish to copy.",
            ),
            Menu::AddPass(ChooseDir) => rofi_mode::String::from(
                "Directory selection, choose one of the existing directories, or type a new custom one.",
            ),
            Menu::AddPass(ChooseFileName) => rofi_mode::String::from(
                "File name input, type the file name you wish to save the password to. It could be the url of the site the password belongs to, or just the name, for example archlinux.org, or just simply archlinux.",
            ),
            Menu::AddPass(ChooseEntryType) => rofi_mode::String::from(
                "Input method selection, choose if you wish to have a password simply generated for you, or if you wish to add an already existing one.",
            ),
            Menu::ChangePass(ChooseFile) => rofi_mode::String::from(
                "Password changing file selection, type the file name of the password you wish to change.",
            ),
            Menu::ChangePass(ChooseNewPassEntryType) => rofi_mode::String::from(
                "Input method selection, choose if you wish to have the new password simply generated for you, or if you wish to change to your own existing one.",
            ),
            _ => rofi_mode::String::new(),
        }
    }
}

export_mode!(ClipKeeperModi);