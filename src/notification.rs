#[cfg(feature = "notify")]
use notify_rust::Notification;

use crate::config::Config;
use crate::library::Library;
use crate::model::playable::Playable;

/// Send a desktop notification for the currently playing track.
#[cfg(feature = "notify")]
pub fn notify_now_playing(cfg: &Config, track: &Playable, library: &Library) {
    let format = cfg
        .values()
        .notification_format
        .clone()
        .unwrap_or_default();
    let default_title = crate::config::NotificationFormat::default().title.unwrap();
    let title = format.title.unwrap_or_else(|| default_title.clone());

    let default_body = crate::config::NotificationFormat::default().body.unwrap();
    let body = format.body.unwrap_or_else(|| default_body.clone());

    let summary_txt = Playable::format(track, &title, library);
    let body_txt = Playable::format(track, &body, library);
    let cover_url = track.cover_url();
    std::thread::spawn(move || send_notification(&summary_txt, &body_txt, cover_url));
}

/// Send a notification using the desktop's default notification method.
///
/// `summary_txt`: A short title for the notification.
/// `body_txt`: The actual content of the notification.
/// `cover_url`: A URL to an image to show in the notification.
#[cfg(feature = "notify")]
fn send_notification(summary_txt: &str, body_txt: &str, cover_url: Option<String>) {
    let mut n = Notification::new();
    n.appname("respot").summary(summary_txt).body(body_txt);

    // album cover image
    if let Some(u) = cover_url {
        let path = crate::utils::cache_path_for_url(u.to_string());
        if !path.exists()
            && let Err(e) = crate::utils::download(u, path.clone())
        {
            log::error!("Failed to download cover: {e}");
        }
        n.icon(path.to_str().unwrap());
    }

    // XDG desktop entry hints
    #[cfg(all(unix, not(target_os = "macos")))]
    n.urgency(notify_rust::Urgency::Low)
        .hint(notify_rust::Hint::Transient(true))
        .hint(notify_rust::Hint::DesktopEntry("respot".into()));

    match n.show() {
        Ok(_handle) => {
            // only available for XDG
            #[cfg(all(unix, not(target_os = "macos")))]
            log::info!("Created notification: {}", _handle.id());
        }
        Err(e) => log::error!("Failed to send notification cover: {e}"),
    }
}
