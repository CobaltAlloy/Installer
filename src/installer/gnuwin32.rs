use std::{fs::File, io::Cursor, path::PathBuf};

const GNU_WIN32_BIN_DOWNLOAD_URL: &str = "https://downloads.sourceforge.net/project/gnuwin32/patch/2.5.9-7/patch-2.5.9-7-bin.zip?ts=gAAAAABl1FheHDGGDzMdv7y0yCJdSoon2i65z-NnrIuL-odvU4C8HmSRl8Xk3W9-bE-k11VZQPDoTjJKQEGbkGuVXPOu3p_ewQ%3D%3D&r=https%3A%2F%2Fsourceforge.net%2Fprojects%2Fgnuwin32%2Ffiles%2Fpatch%2F2.5.9-7%2Fpatch-2.5.9-7-bin.zip%2Fdownload%3Fuse_mirror%3Dnetix%26download%3D";

/// Installs gnuwin32 patch.exe into the path
pub async fn get_win32_patch(base_path: PathBuf) -> Result<(), reqwest::Error> {
    let reqwest_client = reqwest::Client::new();
    let result = reqwest_client
        .get(GNU_WIN32_BIN_DOWNLOAD_URL)
        .send()
        .await?;
    let zip_bytes = result
        .bytes()
        .await?;

    let cursor = Cursor::new(zip_bytes);

    let mut archive = zip::ZipArchive::new(cursor).expect("Failed to read zip archive");

    // We only need the one .exe
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).expect("Failed zip crawl");

        let name = file.enclosed_name().unwrap().to_str().unwrap();

        if name == "bin/patch.exe" {
            let outpath = base_path.join("patch.exe");

            let mut outfile = File::create(&outpath).unwrap();
            std::io::copy(&mut file, &mut outfile).unwrap();

            return Ok(());
        }
    }

    unreachable!("Downloaded patch.zip did not have patch.exe, what?");
}
