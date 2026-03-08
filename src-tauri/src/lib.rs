use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                check_for_updates(handle).await;
            });
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

async fn check_for_updates(app: tauri::AppHandle) {
    let updater = match app.updater() {
        Ok(updater) => updater,
        Err(e) => {
            log::error!("Failed to get updater: {}", e);
            return;
        }
    };

    let update = match updater.check().await {
        Ok(Some(update)) => update,
        Ok(None) => return,
        Err(e) => {
            log::error!("Failed to check for updates: {}", e);
            return;
        }
    };

    let version = update.version.clone();

    // Ask user if they want to update
    let confirmed = app
        .dialog()
        .message(format!(
            "Eine neue Version ({}) ist verfügbar. Jetzt aktualisieren?",
            version
        ))
        .title("Update verfügbar")
        .kind(MessageDialogKind::Info)
        .buttons(MessageDialogButtons::OkCancelCustom(
            "Aktualisieren".into(),
            "Später".into(),
        ))
        .blocking_show();

    if !confirmed {
        return;
    }

    // Download and install
    match update.download_and_install(|_, _| {}, || {}).await {
        Ok(_) => {
            app.dialog()
                .message("Das Update wurde installiert. Die App wird jetzt neu gestartet.")
                .title("Update abgeschlossen")
                .kind(MessageDialogKind::Info)
                .blocking_show();
            app.restart();
        }
        Err(e) => {
            log::error!("Failed to install update: {}", e);
            app.dialog()
                .message(format!("Update fehlgeschlagen: {}", e))
                .title("Fehler")
                .kind(MessageDialogKind::Error)
                .blocking_show();
        }
    }
}
