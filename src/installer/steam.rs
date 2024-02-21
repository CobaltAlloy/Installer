use std::path::PathBuf;

const COBALT_APPID: usize = 357340;

/// Creates the steamappid.txt file
pub async fn create_app_id_txt(base_path: PathBuf) {
    let appid_path = base_path.join("steam_appid.txt");

    std::fs::write(appid_path, COBALT_APPID.to_string()).unwrap();
}
