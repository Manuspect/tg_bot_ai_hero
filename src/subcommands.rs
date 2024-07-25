// Copyright 2018-2022 Cargill Incorporated
// Copyright 2018 Intel Corporation
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#[cfg(feature = "command")]
pub mod command;
pub mod error;
#[cfg(feature = "authorization-handler-maintenance")]
pub mod maintenance;
// pub mod permissions;
#[cfg(feature = "playlist-smallbank")]
pub mod playlist;
#[cfg(feature = "authorization-handler-rbac")]
pub mod rbac;
#[cfg(any(feature = "workload", feature = "playlist-smallbank"))]
mod request_logger;
pub mod time;
#[cfg(feature = "user")]
pub mod user;
#[cfg(feature = "workload")]
pub mod workload;

use std::collections::HashMap;
use std::ffi::CString;
use std::io::{Error as IoError, ErrorKind};
use std::path::Path;

use clap::ArgMatches;

use self::error::CliError;

#[cfg(any(feature = "workload", feature = "playlist-smallbank"))]
const DEFAULT_LOG_TIME_SECS: u32 = 30; // time in seconds

/// A CLI Command Action.
///
/// An Action is a single subcommand for CLI operations.
pub trait Action {
    /// Run a CLI Action with the given args
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches>) -> Result<(), CliError>;
}

/// A collection of Subcommands associated with a single parent command.
#[derive(Default)]
pub struct SubcommandActions<'a> {
    actions: HashMap<String, Box<dyn Action + 'a>>,
}

impl<'a> SubcommandActions<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_command<'action: 'a, A: Action + 'action>(
        mut self,
        command: &str,
        action: A,
    ) -> Self {
        self.actions.insert(command.to_string(), Box::new(action));

        self
    }
}

impl<'s> Action for SubcommandActions<'s> {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches>) -> Result<(), CliError> {
        let args = arg_matches.ok_or(CliError::RequiresArgs)?;

        match args.subcommand() {
            Some((subcommand, args)) => {
                if let Some(action) = self.actions.get_mut(subcommand) {
                    action.run(Some(args))
                } else {
                    Err(CliError::InvalidSubcommand)
                }
            }
            None => Ok({}),
        }
    }
}

fn chown(path: &Path, uid: u32, gid: u32) -> Result<(), CliError> {
    let pathstr = path
        .to_str()
        .ok_or_else(|| CliError::EnvironmentError(format!("Invalid path: {:?}", path)))?;
    let cpath =
        CString::new(pathstr).map_err(|err| CliError::EnvironmentError(format!("{}", err)))?;
    let result = unsafe { libc::chown(cpath.as_ptr(), uid, gid) };
    match result {
        0 => Ok(()),
        code => Err(CliError::EnvironmentError(format!(
            "Error chowning file {}: {}",
            pathstr, code
        ))),
    }
}

fn msg_from_io_error(err: IoError) -> String {
    match err.kind() {
        ErrorKind::NotFound => "File not found".into(),
        ErrorKind::PermissionDenied => "Permission denied".into(),
        ErrorKind::InvalidData => "Invalid data".into(),
        _ => "Unknown I/O error".into(),
    }
}

// Takes a vec of vecs of strings. The first vec should include the title of the columns.
// The max length of each column is calculated and is used as the column with when printing the
// table.
fn print_table(table: Vec<Vec<String>>) {
    let mut max_lengths = Vec::new();

    // find the max lengths of the columns
    for row in table.iter() {
        for (i, col) in row.iter().enumerate() {
            if let Some(length) = max_lengths.get_mut(i) {
                if col.len() > *length {
                    *length = col.len()
                }
            } else {
                max_lengths.push(col.len())
            }
        }
    }

    // print each row with correct column size
    for row in table.iter() {
        let mut col_string = String::from("");
        for (i, len) in max_lengths.iter().enumerate() {
            if let Some(value) = row.get(i) {
                col_string.push_str(value);
                col_string.push_str(&" ".repeat(1 + *len - value.len()));
            } else {
                col_string += &" ".repeat(*len);
            }
        }
        println!("{}", col_string);
    }
}

#[cfg(not(feature = "sqlite"))]
use self::postgres::get_default_database;
#[cfg(feature = "sqlite")]
use build_database::build_database::get_default_database;

pub struct MigrateAction;

impl Action for MigrateAction {
    fn run<'a>(&mut self, arg_matches: Option<&ArgMatches>) -> Result<(), CliError> {
        let url = if let Some(args) = arg_matches {
            match args.get_one::<String>("connect") {
                Some(url) => url.to_owned(),
                None => get_default_database()?,
            }
        } else {
            get_default_database()?
        };

        Ok(())
    }
}
