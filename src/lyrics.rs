use std::time::Duration;

use log::debug;

use crate::model::playable::Playable;

/// A single lyrics line with its start time.
pub struct LyricLine {
    pub start: Duration,
    pub text: String,
}

/// Fetched lyrics for a track.
pub struct Lyrics {
    pub lines: Vec<LyricLine>,
    pub synced: bool,
}

/// Fetch lyrics for the currently playing track from LRCLIB.
pub fn fetch(playable: &Playable) -> Option<Lyrics> {
    fetch_lrclib(playable)
}

/// Fetch lyrics from LRCLIB.
fn fetch_lrclib(playable: &Playable) -> Option<Lyrics> {
    let track = match playable {
        Playable::Track(t) => t,
        _ => return None,
    };
    let artist = track.artists.first()?;
    let title = &track.title;
    let album = track.album.as_deref();
    let duration_secs = (track.duration / 1000) as u64;

    let mut url = format!(
        "https://lrclib.net/api/get?artist_name={}&track_name={}&duration={}",
        urlencoded(artist),
        urlencoded(title),
        duration_secs,
    );
    if let Some(album) = album {
        url.push_str(&format!("&album_name={}", urlencoded(album)));
    }

    debug!("fetching lyrics from LRCLIB: {url}");

    let response = reqwest::blocking::Client::new()
        .get(&url)
        .header("User-Agent", "respot/1.0")
        .send()
        .ok()?;

    if !response.status().is_success() {
        debug!("LRCLIB returned status {}", response.status());
        return None;
    }

    let body: serde_json::Value = response.json().ok()?;

    // Prefer synced lyrics (with LRC timestamps) for full parity with Spotify lyrics
    if let Some(synced_text) = body["syncedLyrics"].as_str() {
        let lines = parse_lrc(synced_text);
        if !lines.is_empty() {
            debug!("fetched {} synced lyrics lines from LRCLIB", lines.len());
            return Some(Lyrics { lines, synced: true });
        }
    }

    // Fall back to plain lyrics (no sync, no seek)
    if let Some(plain_text) = body["plainLyrics"].as_str() {
        let lines: Vec<LyricLine> = plain_text
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| LyricLine {
                start: Duration::ZERO,
                text: l.to_string(),
            })
            .collect();
        if !lines.is_empty() {
            debug!("fetched {} plain lyrics lines from LRCLIB", lines.len());
            return Some(Lyrics { lines, synced: false });
        }
    }

    None
}

/// Parse LRC format: "[mm:ss.xx] text"
fn parse_lrc(text: &str) -> Vec<LyricLine> {
    text.lines()
        .filter_map(|line| {
            let close = line.find(']')?;
            let timestamp = &line[1..close];
            let text = line[close + 1..].trim();
            if text.is_empty() {
                return None;
            }
            let mut parts = timestamp.split(':');
            let minutes: u64 = parts.next()?.parse().ok()?;
            let sec_str = parts.next()?;
            let mut sec_parts = sec_str.split('.');
            let seconds: u64 = sec_parts.next()?.parse().ok()?;
            let frac: u64 = sec_parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
            // Handle both 2-digit centiseconds and 3-digit milliseconds
            let frac_ms = if sec_str.split('.').nth(1).map(|s| s.len()).unwrap_or(0) >= 3 {
                frac
            } else {
                frac * 10
            };
            let ms = minutes * 60_000 + seconds * 1000 + frac_ms;
            Some(LyricLine {
                start: Duration::from_millis(ms),
                text: text.to_string(),
            })
        })
        .collect()
}

fn urlencoded(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}
