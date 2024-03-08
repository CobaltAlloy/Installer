use cfg_if::cfg_if;
use colored::Colorize;
use fs_extra::dir::CopyOptions;
use spinners::{Spinner, Spinners};
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use crate::installer::inquire::FilePathCompleter;
use crate::installer::inquire::InquireGamePathValidator;
use crate::installer::inquire::InquirePathDoesntExistValidator;
use crate::installer::inquire::InquirePathExistsValidator;
use crate::installer::windows::exit_or_windows;
use crate::installer::INSTALLER_FOLDER;

pub mod installer;

pub extern crate serde;
pub extern crate serde_json;

#[tokio::main]
async fn main() {
    println!("");
    println!("┏━┓╻  ╻  ┏━┓╻ ╻   ╻┏┓╻┏━┓╺┳╸┏━┓╻  ╻  ┏━╸┏━┓");
    println!("┣━┫┃  ┃  ┃ ┃┗┳┛   ┃┃┗┫┗━┓ ┃ ┣━┫┃  ┃  ┣╸ ┣┳┛");
    println!("╹ ╹┗━╸┗━╸┗━┛ ╹    ╹╹ ╹┗━┛ ╹ ╹ ╹┗━╸┗━╸┗━╸╹┗╸");
    println!("");

    let mut cobalt_dir: Option<PathBuf> = None;

    let cobalt_directory_found = installer::cobalt::find_cobalt_path();

    if let Some(dir) = cobalt_directory_found {
        let prompt = format!("Found Cobalt at {}, is that right?", dir.display());
        let confirm = inquire::Confirm::new(&prompt).with_default(true).prompt();
        if confirm.expect("Cancelled") {
            cobalt_dir = Some(dir);
        }
    }

    if cobalt_dir.is_none() {
        let prompt = "Please enter your Cobalt game folder path:";

        let path = inquire::Text::new(prompt)
            .with_validator(InquireGamePathValidator {})
            .with_autocomplete(FilePathCompleter::default())
            .prompt()
            .unwrap();
        cobalt_dir = Some(path.into());
    }

    println!("");
    println!(
        "It is {} recommended you create a new copy of Cobalt for Alloy.",
        "highly".italic()
    );
    println!("Installing to your main copy might break your game");
    let mut create_new_copy = inquire::Confirm::new("Create a new copy and install there?")
        .with_default(true)
        .prompt()
        .unwrap();

    if !create_new_copy {
        create_new_copy = !inquire::Confirm::new("Are you sure?")
            .with_default(false)
            .prompt()
            .unwrap();

        if create_new_copy {
            println!("Okay, I will create a new copy");
        }
    }

    let mut install_dir: Option<PathBuf> = None;

    if !create_new_copy {
        println!("Okay, if you say so...");
        install_dir = cobalt_dir.clone();
    }

    if create_new_copy {
        let mut parent_dir = cobalt_dir.clone().unwrap();
        parent_dir = parent_dir.parent().unwrap().to_path_buf();

        let mut copy_dir = parent_dir.join("CobaltAlloy");

        // If we already have a CobaltAlloy
        let mut alloy_n: u8 = 1;
        while copy_dir.exists() {
            copy_dir = parent_dir.join(format!("CobaltAlloy{}", alloy_n));
            alloy_n += 1;

            if alloy_n > 200 {
                print!("I give up, you have too many installations of Alloy...");
                std::process::exit(42);
            }
        }

        let prompt = format!(
            "Is {} okay? (for the new copy of Cobalt)",
            copy_dir.display()
        );
        let copy_dir_ok = inquire::Confirm::new(&prompt)
            .with_default(true)
            .prompt()
            .unwrap();

        if !copy_dir_ok {
            let prompt = "Okay, where should I create a new copy then?";
            copy_dir = inquire::Text::new(prompt)
                .with_validator(InquirePathDoesntExistValidator {})
                .with_autocomplete(FilePathCompleter::default())
                .prompt()
                .unwrap()
                .into();
        }

        let msg = format!("Creating a new copy of Cobalt at {}...", copy_dir.display());
        let mut sp = Spinner::new(Spinners::Dots, msg);

        std::fs::create_dir_all(copy_dir.clone()).unwrap();

        let options = CopyOptions {
            copy_inside: true,
            content_only: true,
            ..Default::default()
        };

        fs_extra::dir::copy(cobalt_dir.clone().unwrap(), copy_dir.clone(), &options).unwrap();

        sp.stop_with_message(format!(
            "Created new copy of Cobalt at {}!",
            copy_dir.display()
        ));

        install_dir = Some(copy_dir);
    } 

    println!("");
    println!("One last thing: I need a decompiled daisyMoon folder.");
    println!("You can either decompile it yourself, or you can download it from the Cobalt Archive:");
    println!("(https://drive.google.com/drive/folders/1jasI5F9X8kWauTzx3fT-qy6_aMJZx_fi)");
    println!("");
    println!(
        "Once you have that ready, give me either the path to a zip or a full daisyMoon folder."
    );

    let mut daisymoon_folder_path: Option<PathBuf> = None;

    while daisymoon_folder_path.is_none() {
        let path: PathBuf = inquire::Text::new("daisyMoon path (.zip or folder)")
            .with_validator(InquirePathExistsValidator {})
            .with_autocomplete(FilePathCompleter::default())
            .prompt()
            .unwrap()
            .into();

        if path.is_dir() {
            let contains_right_file = path
                .clone()
                .join("lib/manipulators/galaxyManipulators.lua")
                .exists();

            if !contains_right_file {
                println!("That folder doesn't have the right files, please try again.");
                continue;
            }

            let mut sp = Spinner::new(Spinners::Dots, "Creating daisyMoon folder...".into());

            let daisy_path = install_dir.clone().unwrap().join("daisyMoon");

            std::fs::create_dir_all(daisy_path.clone()).unwrap();

            let options = CopyOptions {
                copy_inside: true,
                content_only: true,
                overwrite: true, // We might be copying an entire daisyMoon folder from another alloy install
                ..Default::default()
            };

            fs_extra::dir::copy(path, daisy_path.clone(), &options).unwrap();

            sp.stop_with_message("Created daisyMoon folder!".into());

            daisymoon_folder_path = Some(daisy_path);
        } else if let Some(extension) = path.extension() {
            if extension != "zip" {
                println!("That path is not a folder or zip; please try again.");
                continue;
            }

            let mut sp = Spinner::new(Spinners::Dots, "Creating daisyMoon folder...".into());

            let bytes = std::fs::read(path).unwrap();

            // Decompress the zip into the daisymoon folder
            let cursor = Cursor::new(bytes);

            let mut archive = zip::ZipArchive::new(cursor).expect("Failed to read zip archive");

            for i in 0..archive.len() {
                let mut file = archive.by_index(i).expect("Failed zip crawl");

                let name = file
                    .enclosed_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace("cobalt/", "") // Fix potentially downloading it as a zip from the wrong folders
                    .replace("daisyMoon/", "");

                let outpath = install_dir.clone().unwrap().join("daisyMoon").join(name);

                std::fs::create_dir_all(outpath.clone().parent().unwrap()).unwrap();

                let mut outfile = File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }

            sp.stop_with_message("Created daisyMoon folder!".into());

            daisymoon_folder_path = Some(install_dir.clone().unwrap().join("daisyMoon"));
        } else {
            println!("That path is not a folder or zip, please try again.");
            continue;
        }
    }

    println!("Installing to {}...", install_dir.clone().unwrap().display());

    installer::steam::create_app_id_txt(install_dir.clone().unwrap()).await;
    println!("Created appid!");

    let mut sp = Spinner::new(Spinners::Dots, "Downloading Alloy...".into());

    if let Err(e) = std::fs::create_dir_all(install_dir.clone().unwrap().join(INSTALLER_FOLDER)) {
        println!("Failed to create installer file directory: {}", e);
        exit_or_windows(102);
    }

    let alloy_dl_result =
        installer::alloy::download_alloy_files(install_dir.clone().unwrap()).await;
    if let Err(e) = alloy_dl_result {
        println!("Failed to download Alloy with error: {}", e);
        println!("Are you connected to the internet?");
        exit_or_windows(2);
    }

    sp.stop_with_message("Downloaded Alloy!".into());

    cfg_if! {
        if #[cfg(target_os = "windows")] {
            println!("Since you're running Windows, I'll need to download patch.exe");

            let mut sp = Spinner::new(Spinners::Dots, "Downloading patch...".into());

            let patch_dl_result = installer::gnuwin32::get_win32_patch(install_dir.clone().unwrap()).await;
            if let Err(e) = patch_dl_result {
                println!("Failed to download patch with error: {}", e);
                println!("Are you connected to the internet?");
                exit_or_windows(2);
            }

            sp.stop_with_message("Downloaded!".into());
        }
    }

    let mut sp = Spinner::new(
        Spinners::Dots,
        "Syncing line endings with your system...".into(),
    );
    installer::alloy::fix_line_endings(install_dir.clone().unwrap());
    sp.stop_with_message("Synced line endings!".into());

    println!("Running patch!");
    installer::alloy::patch_daisy_with_alloy(install_dir.clone().unwrap()).await;
    println!("Successfully patched!");
    
    println!("Writing metadata to make future updating easier...");
    installer::metadata::write_metadata(install_dir.clone().unwrap());
    println!("Done!");
    
    if let Err(e) = std::fs::create_dir_all(install_dir.clone().unwrap().join("alloys")) {
        println!("Failed to create alloys directory: {}", e);
        exit_or_windows(99);
    }
    
    println!("");
    println!("{}", "Successfully installed Alloy!".bold());
    println!(
        "Add {} to Steam as a non-steam game and enjoy! :D",
        install_dir.unwrap().join("cobaltDM.exe").display()
    );

    exit_or_windows(0);
}
