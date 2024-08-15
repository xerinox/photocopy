#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui;

use std::{
    collections::HashSet,
    error::Error,
    fmt::{Debug, Display, Formatter},
};

use bitvec::prelude::*;

use windows::Win32::{Foundation::GetLastError, Storage::FileSystem::GetLogicalDrives};

pub enum GetLogicalDrivesError {
    TooManyDrivesError,
    ApiError(u32),
}

impl Display for GetLogicalDrivesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl Debug for GetLogicalDrivesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            GetLogicalDrivesError::TooManyDrivesError => write!(f, "TooManyDrive"),
            GetLogicalDrivesError::ApiError(code) => write!(f, "ApiError{code}"),
        }
    }
}

impl Error for GetLogicalDrivesError {}

const INVALID_DRIVE_LETTER_BITMASK: u32 = 0b11111100_00000000_00000000_00000000;

/// https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getlogicaldrives
pub fn get_drives() -> Result<HashSet<char>, GetLogicalDrivesError> {
    let drives_bitmap = unsafe { GetLogicalDrives() };

    // If the function fails, the return value is zero. To get extended error information, call GetLastError.
    if drives_bitmap == 0 {
        let err = unsafe { GetLastError() };
        Err(GetLogicalDrivesError::ApiError(err.0))
    } else if drives_bitmap & INVALID_DRIVE_LETTER_BITMASK != 0 {
        Err(GetLogicalDrivesError::TooManyDrivesError)
    } else {
        Ok(drives_bitmap
            .view_bits::<Lsb0>()
            .iter()
            .zip('A'..='Z')
            .filter_map(|(bit, drive_letter)| {
                // a bit derefs into a bool
                if *bit {
                    Some(drive_letter)
                } else {
                    None
                }
            })
            .collect())
    }
}

pub struct MyApp {
    drives: Result<HashSet<char>, GetLogicalDrivesError>,
    drive_letters: Vec<String>,
    selected_drive: Option<usize>,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        MyApp {
            drives: get_drives(),
            drive_letters: MyApp::convert_drives_to_drive_letters(get_drives()),
            selected_drive: None,
        }
    }
    fn convert_drives_to_drive_letters(res: Result<HashSet<char>, GetLogicalDrivesError>) -> Vec<String> {
        let mut output:Vec<String> = vec!();
        if let Ok(drives) = res {
            drives.iter().map(|l| {
               format!("{l}:/")
            }).for_each(|s|{output.push(s)});
        }
        output
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.drives.is_ok() {
            if let Some(selected_drive) = self.selected_drive {
                dbg!(&self.drive_letters[selected_drive]);
            }
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Try to close the window");
            let mut selected = match self.selected_drive {
                Some(selected_drive_index) => String::from(&self.drive_letters[selected_drive_index]),
                None => String::from("None selected")
            };
            let drive_letters = self.drive_letters.clone();

            if let Ok(_drives) = &self.drives {
                egui::ComboBox::from_label("Select phone drive letter")
                    .selected_text(format!("{:?}", selected))
                    .show_ui(ui, |ui| {
                        for (index, drive) in drive_letters.iter().enumerate() {
                            let val = ui.selectable_value(&mut selected, String::from(&self.drive_letters[index]), drive);
                            if val.clicked() {
                                self.selected_drive = Some(index);
                            }
                            
                        }
                    });
            }
        });

    }
}
