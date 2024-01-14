#![feature(iter_intersperse)]
// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

extern crate google_drive3 as drive;
pub mod description;
pub mod drive_hub;
pub mod empty_file;

use crate::drive_hub::DriveHub;
use anyhow::Context;
use anyhow::Result;
use eframe::egui;
use notify_rust::Notification;
use rfd::FileDialog;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<()> {
    let cache_dir = &dirs::cache_dir()
        .context("Failed to get cache dir")?
        .join("document-upload-cli");
    let mut drive_hub = DriveHub::new(cache_dir).await?;

    let options = eframe::NativeOptions::default();

    let mut folders: Vec<PathBuf> = vec![];

    eframe::run_simple_native("Test", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.group(|ui| {
                ui.heading("Documents list:");
                ui.label(
                    folders
                        .iter()
                        .map(|path| path.to_str().unwrap())
                        .intersperse("\n")
                        .collect::<String>(),
                );
            });
            if ui.button("choose documents").clicked() {
                if let Some(choosen_folders) = FileDialog::new()
                    .set_title("Choose files to open")
                    .pick_folders()
                {
                    folders = choosen_folders
                }
            }
            if ui.button("upload").clicked() {
                let mut uploaded_document_count = 0;
                for folder in &folders {
                    if drive_hub.upload_document_blocking(folder).is_ok() {
                        uploaded_document_count += 1;
                        continue;
                    }

                    Notification::new()
                        .summary("Upload failed")
                        .body(&format!(
                            "Failed to upload document {}",
                            folder.to_str().unwrap()
                        ))
                        .timeout(0)
                        .show()
                        .unwrap();
                }

                Notification::new()
                    .summary("Upload summary")
                    .body(&format!("{} document uploaded", uploaded_document_count))
                    .timeout(0)
                    .show()
                    .unwrap();
            }
        });
    })
    .unwrap();
    Ok(())
}
