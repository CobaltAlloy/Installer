use cfg_if::cfg_if;
use colored::Colorize;
use fs_extra::dir::CopyOptions;
use std::fs::File;
use std::io::Cursor;
use std::path::PathBuf;

use crate::installer::cobalt::InquireGamePathValidator;
use crate::installer::cobalt::InquirePathDoesntExistValidator;
use crate::installer::cobalt::InquirePathExistsValidator;
use crate::installer::windows::exit_or_windows;

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
        if confirm.expect("Invalid input") {
            cobalt_dir = Some(dir);
        }
    }

    if cobalt_dir.is_none() {
        let prompt = "Please enter your Cobalt game folder path";

        let path = inquire::Text::new(prompt)
            .with_validator(InquireGamePathValidator {})
            .prompt()
            .unwrap();
        cobalt_dir = Some(path.into());
    }

    println!("");
    println!(
        "It is {} recommended to create a new copy of Cobalt for Alloy.",
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
        println!("Okay, if you say so..");
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
                .prompt()
                .unwrap()
                .into();
        }

        println!("Creating a new copy of Cobalt at {}..", copy_dir.display());

        std::fs::create_dir_all(copy_dir.clone()).unwrap();

        let options = CopyOptions {
            copy_inside: true,
            content_only: true,
            ..Default::default()
        };

        fs_extra::dir::copy(cobalt_dir.clone().unwrap(), copy_dir.clone(), &options).unwrap();

        install_dir = Some(copy_dir);
    }

    println!("");
    println!("One last thing, I need a decompiled daisyMoon folder.");
    println!("You can either decompile it yourself or you can download it from the Cobalt Archive");
    println!("(https://drive.google.com/drive/folders/1jasI5F9X8kWauTzx3fT-qy6_aMJZx_fi)");
    println!("");
    println!(
        "Once you have that ready, give me either the path to a zip or a full daisyMoon folder."
    );

    let mut daisymoon_folder_path: Option<PathBuf> = None;

    while daisymoon_folder_path.is_none() {
        let path: PathBuf = inquire::Text::new("daisyMoon path (.zip or folder)")
            .with_validator(InquirePathExistsValidator {})
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

            let daisy_path = install_dir.clone().unwrap().join("daisyMoon");

            std::fs::create_dir_all(daisy_path.clone()).unwrap();

            let options = CopyOptions {
                copy_inside: true,
                content_only: true,
                overwrite: true, // We might be copying an entire daisyMoon folder from another
                // Alloy install
                ..Default::default()
            };

            fs_extra::dir::copy(path, daisy_path.clone(), &options).unwrap();

            daisymoon_folder_path = Some(daisy_path);
        } else if let Some(extension) = path.extension() {
            if extension != "zip" {
                println!("That path is not a folder or zip, please try again.");
                continue;
            }

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
                    .replace("daisyMoon/", "");

                let outpath = install_dir.clone().unwrap().join("daisyMoon").join(name);

                std::fs::create_dir_all(outpath.clone().parent().unwrap()).unwrap();

                let mut outfile = File::create(&outpath).unwrap();
                std::io::copy(&mut file, &mut outfile).unwrap();
            }

            daisymoon_folder_path = Some(install_dir.clone().unwrap().join("daisyMoon"));
        } else {
            println!("That path is not a folder or zip, please try again.");
            continue;
        }
    }

    println!("Successfully created daisyMoon folder!");
    println!("");
    println!("Installing to {}..", install_dir.clone().unwrap().display());

    println!("Creating appid...");
    installer::steam::create_app_id_txt(install_dir.clone().unwrap()).await;

    println!("Downloading Alloy..");
    installer::alloy::download_alloy_files(install_dir.clone().unwrap())
        .await
        .unwrap();

    cfg_if! {
        if #[cfg(target_os = "windows")] {
            println!("Since you're running windows, I'll need to install patch.exe");
            installer::gnuwin32::get_win32_patch(install_dir.clone().unwrap()).await.unwrap();
            println!("Installed!");
        }
    }

    println!("Just in case, syncing line endings before running patch..");
    installer::alloy::fix_line_endings(install_dir.clone().unwrap());
    println!("Done!");

    println!("Running patch command..");
    installer::alloy::patch_daisy_with_alloy(install_dir.clone().unwrap()).await;

    println!("");
    println!("{}", "Successfully installed Alloy!".bold());
    println!(
        "Add {} to Steam as a non-steam game and enjoy! :D",
        install_dir.unwrap().join("cobaltDM.exe").display()
    );

    exit_or_windows(0);
}
