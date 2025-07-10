// Copyright (c) 2025 Hemashushu <hippospark@gmail.com>, All rights reserved.
//
// This Source Code Form is subject to the terms of
// the Mozilla Public License version 2.0 and additional exceptions.
// For more details, see the LICENSE, LICENSE.additional, and CONTRIBUTING files.

use std::fs::File;

use regex_anre::{context::MatchRange, Regex};

pub struct ThreadResources {
    regexes: Vec<Option<Regex>>,

    last_captures: Vec<MatchRange>,

    // Files that are opened by the thread.
    // These files are used for reading and writing data.
    // The first three files are standard input, output, and error.
    // Their indices are 0, 1, and 2 respectively.
    files: Vec<Option<FileObject>>,
}

pub enum FileObject {
    StdIn,
    StdOut,
    StdErr,
    User(File),
}

impl ThreadResources {
    pub fn new() -> Self {
        Self {
            regexes: Vec::new(),
            last_captures: Vec::new(),
            files: vec![
                Some(FileObject::StdIn),
                Some(FileObject::StdOut),
                Some(FileObject::StdErr),
            ],
        }
    }

    /// Adds a new regex to the first `None` slot in the `regexes` vector.
    /// Returns the index of the added regex.
    pub fn add_regex(&mut self, regex: Regex) -> usize {
        if let Some(index) = self.regexes.iter().position(Option::is_none) {
            self.regexes[index] = Some(regex);
            index
        } else {
            // If no None slot is found, push the regex to the end of the vector.
            self.regexes.push(Some(regex));
            self.regexes.len() - 1
        }
    }

    pub fn get_regex(&self, index: usize) -> Option<&Regex> {
        self.regexes.get(index).and_then(Option::as_ref)
    }

    pub fn set_last_captures(&mut self, captures: Vec<MatchRange>) {
        self.last_captures = captures;
    }

    pub fn get_last_captures(&self) -> &[MatchRange] {
        &self.last_captures
    }

    pub fn remove_regex(&mut self, index: usize) {
        if index < self.regexes.len() {
            self.regexes[index] = None;
        }
    }

    /// Add a new user-opened file to the first `None` slot in the `files` vector.
    /// Returns the index of the added file.
    pub fn add_file(&mut self, file: FileObject) -> usize {
        if let Some(index) = self.files.iter().position(Option::is_none) {
            self.files[index] = Some(file);
            index
        } else {
            // If no None slot is found, push the file to the end of the vector.
            self.files.push(Some(file));
            self.files.len() - 1
        }
    }

    pub fn get_file(&self, index: usize) -> Option<&FileObject> {
        self.files.get(index).and_then(Option::as_ref)
    }

    pub fn remove_file(&mut self, index: usize) {
        if index < self.files.len() {
            self.files[index] = None;
        }
    }
}
