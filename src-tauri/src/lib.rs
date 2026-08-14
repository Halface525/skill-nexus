mod core;

#[tauri::command]
fn load_skills() -> Result<Vec<core::Skill>, String> {
    core::load_skills()
}

#[tauri::command]
fn read_skill_md(dir: String, lang: String) -> Result<String, String> {
    core::read_skill_md(&dir, &lang)
}

#[tauri::command]
fn install_skill(src: String, lang: String) -> Result<String, String> {
    core::install_skill(&src, &lang)
}

#[tauri::command]
fn sync_all() -> Result<core::SyncResult, String> {
    core::sync_all()
}

#[tauri::command]
fn scan_info() -> core::ScanInfo {
    core::scan_info()
}

#[tauri::command]
fn get_settings() -> core::SettingsView {
    core::get_settings()
}

#[tauri::command]
fn set_unified_library(path: String) -> Result<(), String> {
    core::set_unified_library(&path)
}

#[tauri::command]
fn open_dir(path: String) -> Result<(), String> {
    core::open_dir(&path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            load_skills,
            read_skill_md,
            install_skill,
            sync_all,
            scan_info,
            get_settings,
            set_unified_library,
            open_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
