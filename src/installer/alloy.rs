use std::{fs, path::PathBuf};

use cfg_if::cfg_if;

#[allow(unused_imports)]
use newline_converter::{dos2unix, unix2dos};
use reqwest::Error;
use tokio::process::Command;

use crate::installer::windows::exit_or_windows;

const TRANSLATIONS_FILE_URL: &str =
    "https://raw.githubusercontent.com/Creeper-boop/Alloy/master/alloy/eng.translations";
#[allow(dead_code)]
const LATEST_WINDOWS_DIFF_URL: &str = "https://raw.githubusercontent.com/Creeper-boop/Alloy/master/alloy/win/alloy_editor_mod_0_0_3_win.diff";
#[allow(dead_code)]
const LATEST_LINUX_DIFF_URL: &str = "https://raw.githubusercontent.com/Creeper-boop/Alloy/master/alloy/lin/alloy_editor_mod_0_0_3_lin.diff";

/// The name of the diff saved when downloading
const SAVED_DIFF_NAME: &str = "alloy_editor_mod.diff";

/// Downloads the required alloy files into the right folders
pub async fn download_alloy_files(base_path: PathBuf) -> Result<(), Error> {
    let client = reqwest::Client::new();

    let translations = client
        .get(TRANSLATIONS_FILE_URL)
        .send()
        .await?
        .bytes()
        .await?;

    std::fs::write(
        base_path.join("translations").join("eng.translations"),
        translations,
    )
    .unwrap();

    #[cfg(target_os = "windows")]
    let diff_url = LATEST_WINDOWS_DIFF_URL;

    #[cfg(not(target_os = "windows"))]
    let diff_url = LATEST_LINUX_DIFF_URL;

    let diff = client.get(diff_url).send().await?.bytes().await?;

    std::fs::write(base_path.join(SAVED_DIFF_NAME), diff).unwrap();

    Ok(())
}

/// Runs the patch command for the downloaded diff
pub async fn patch_daisy_with_alloy(base_path: PathBuf) {
    let diff_path = base_path.clone().join(SAVED_DIFF_NAME);

    #[cfg(target_os = "windows")]
    let diff_command = format!(
        ".\\patch.exe --ignore-whitespace -p0 -i .\\{}",
        SAVED_DIFF_NAME
    );

    #[cfg(not(target_os = "windows"))]
    let diff_command = format!(
        r#"patch --ignore-whitespace -p0 < "{}""#,
        diff_path.display()
    );

    cfg_if! {
        if #[cfg(target_os = "windows")] {
            let mut command = Command::new("cmd");
            command.current_dir(base_path.clone());
            command.args(["/C", &diff_command]);
        }
        else {
            let mut command = Command::new("sh");
            command.current_dir(base_path.clone());
            command.args(["-c", &diff_command]);
        }
    }

    let output_res = command.output().await;

    if let Err(e) = &output_res {
        println!("Failed to run patch: {}", e);
        exit_or_windows(4);
    }

    let output = output_res.unwrap();

    let mut stdout = String::from_utf8(output.stdout).unwrap();

    // FIXME: This is a very bad hack to ignore the random error of failing to patch
    // launcher.lua with one single print.
    //
    // Why does this happen? Idk, but it isn't *that* important
    stdout = stdout.replace(
        "patching file daisyMoon/launcher.lua
Hunk #1 FAILED at 1.
1 out of 1 hunk FAILED -- saving rejects to file daisyMoon/launcher.lua.rej",
        "",
    );

    let stderr = String::from_utf8(output.stderr).unwrap();

    if stdout.contains("FAILED") {
        println!("Some patch hunks failed!");

        if stdout.contains("different line endings") {
            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
            println!(
                "Patch failed because of different line endings, even though we've converted them"
            );
            println!("Please open an issue on github.");
            exit_or_windows(6);
        } else {
            println!("stdout: {}", stdout);
            println!("stderr: {}", stderr);
            println!("I'm not sure what went wrong");
            println!("Please open an issue on github.");
            exit_or_windows(7);
        }
    }

    if !base_path.clone().join("daisyMoon/alloy.lua").exists() {
        println!("stdout: {}", stdout);
        println!("stderr: {}", stderr);
        println!("Patch command: {}", diff_command.clone());
        println!("Patch catastrophically failed! Please open an issue on github.");
        exit_or_windows(5);
    }
}

// Just in case, fix all the line endings for all files before patching
pub fn fix_line_endings(base_path: PathBuf) {
    let diff_path = base_path.clone().join(SAVED_DIFF_NAME);

    let mut alloy_patch = std::fs::read_to_string(diff_path).unwrap();

    cfg_if! {
        if #[cfg(target_os = "windows")] {
            alloy_patch = unix2dos(&alloy_patch).to_string();
        }
        else {
            alloy_patch = dos2unix(&alloy_patch).to_string();
        }
    }

    std::fs::write(base_path.clone().join(SAVED_DIFF_NAME), alloy_patch).unwrap();

    let base_daisy_path = base_path.clone().join("daisyMoon");

    for entry_res in fs::read_dir(base_daisy_path).unwrap() {
        if let Ok(entry) = entry_res {
            if entry.file_name().to_str().unwrap().contains(".lua") {
                let string_content_res = std::fs::read_to_string(entry.path());

                if let Err(e) = &string_content_res {
                    println!("Failed to read file {}: {}", entry.path().display(), e);
                    exit_or_windows(3);
                }

                let mut string_content = string_content_res.unwrap();

                cfg_if! {
                    if #[cfg(target_os = "windows")] {
                        string_content = unix2dos(&string_content).to_string();
                    }
                    else {
                        string_content = dos2unix(&string_content).to_string();
                    }
                }

                let write_res = std::fs::write(entry.path(), string_content);

                if let Err(e) = write_res {
                    println!("Failed to read file {}: {}", entry.path().display(), e);
                    exit_or_windows(3);
                }
            }
        }
    }
}
