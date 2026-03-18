use tauri::{LogicalPosition, LogicalSize, WebviewBuilder, WebviewUrl, WindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_updater::UpdaterExt;

const TITLEBAR_HEIGHT: f64 = 32.0;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let width = 1280.0_f64;
            let height = 800.0_f64;

            let window = WindowBuilder::new(app, "main")
                .title("Blutmond Loot Council")
                .inner_size(width, height)
                .decorations(false)
                .build()?;

            let _titlebar = window.add_child(
                WebviewBuilder::new("titlebar", WebviewUrl::App("titlebar.html".into())),
                LogicalPosition::new(0.0, 0.0),
                LogicalSize::new(width, TITLEBAR_HEIGHT),
            )?;

            let _content = window.add_child(
                WebviewBuilder::new(
                    "content",
                    WebviewUrl::External(
                        "https://blutmondlc.apps.schmorty.com/".parse().unwrap(),
                    ),
                ),
                LogicalPosition::new(0.0, TITLEBAR_HEIGHT),
                LogicalSize::new(width, height - TITLEBAR_HEIGHT),
            )?;

            // Resize webviews when window resizes
            let tb = _titlebar.clone();
            let ct = _content.clone();
            let win = window.clone();
            window.on_window_event(move |event| {
                if let tauri::WindowEvent::Resized(phys) = event {
                    let scale = win.scale_factor().unwrap_or(1.0);
                    let w = phys.width as f64 / scale;
                    let h = phys.height as f64 / scale;
                    let _ = tb.set_size(LogicalSize::new(w, TITLEBAR_HEIGHT));
                    let _ = ct.set_size(LogicalSize::new(w, (h - TITLEBAR_HEIGHT).max(0.0)));
                }
            });

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
