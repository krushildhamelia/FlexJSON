use flexjson_core::{parse, ParseResult};

#[tauri::command]
fn parse_input(input: String) -> ParseResult {
    parse(&input)
}

#[tauri::command]
async fn parse_file(path: String) -> Result<ParseResult, String> {
    let content = tokio::fs::read_to_string(path).await.map_err(|e| e.to_string())?;
    Ok(parse(&content))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .plugin(
      tauri_plugin_log::Builder::default()
        .level(if cfg!(debug_assertions) { log::LevelFilter::Info } else { log::LevelFilter::Error })
        .build()
    )
    .invoke_handler(tauri::generate_handler![parse_input, parse_file])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
