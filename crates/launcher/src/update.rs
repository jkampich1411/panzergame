//! Releases von GitHub holen, das passende Spielpaket herunterladen, per SHA-256 prüfen,
//! entpacken und als installierte Version merken. Läuft im Hintergrund-Thread.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

/// Neuestes Release (GitHub-API). Mit PANZER_FEED lässt sich ein anderer Feed angeben
/// (z. B. ein lokaler Testserver).
pub const DEFAULT_FEED: &str = "https://api.github.com/repos/martinkopoky-wq/panzergame/releases/latest";
pub const CHECKSUMS: &str = "SHA256SUMS.txt";

#[derive(Clone, Debug)]
pub struct Asset {
    pub name: String,
    pub url: String,
    pub size: u64,
}

#[derive(Clone, Debug)]
pub struct Release {
    pub tag: String,
    pub name: String,
    pub body: String,
    pub date: String,
    pub assets: Vec<Asset>,
}

impl Release {
    pub fn asset(&self, name: &str) -> Option<&Asset> {
        self.assets.iter().find(|a| a.name == name)
    }
}

#[derive(Clone, Debug)]
pub struct Installed {
    pub version: String,
    pub dir: PathBuf,
}

impl Installed {
    pub fn exe(&self) -> PathBuf {
        self.dir.join(if cfg!(windows) { "panzer.exe" } else { "panzer" })
    }
}

pub fn feed_url() -> String {
    std::env::var("PANZER_FEED").unwrap_or_else(|_| DEFAULT_FEED.to_string())
}

/// Name des Spielpakets für dieses System
pub fn platform_asset() -> String {
    let arch = match std::env::consts::ARCH {
        "aarch64" => "aarch64",
        _ => "x86_64",
    };
    match std::env::consts::OS {
        "windows" => "panzer-game-windows-x86_64.zip".into(), // ARM-Windows emuliert x86_64
        "macos" => "panzer-game-macos-universal.zip".into(),
        _ => format!("panzer-game-linux-{arch}.zip"),
    }
}

fn agent() -> ureq::Agent {
    ureq::AgentBuilder::new()
        .timeout_connect(std::time::Duration::from_secs(8))
        .timeout_read(std::time::Duration::from_secs(30))
        .user_agent(concat!("panzer-launcher/", env!("CARGO_PKG_VERSION")))
        .build()
}

pub fn fetch_latest(feed: &str) -> Result<Release, String> {
    let resp = agent().get(feed).set("Accept", "application/vnd.github+json").call();
    let text = match resp {
        Ok(r) => r.into_string().map_err(|e| e.to_string())?,
        Err(ureq::Error::Status(404, _)) => return Err("Es gibt noch kein veröffentlichtes Release.".into()),
        Err(ureq::Error::Status(403, _)) => {
            return Err("GitHub hat zu viele Anfragen abgelehnt, bitte später nochmal.".into())
        }
        Err(ureq::Error::Status(code, _)) => return Err(format!("Server antwortet mit Fehler {code}.")),
        Err(e) => return Err(format!("Keine Verbindung: {e}")),
    };
    parse_release(&text)
}

pub fn parse_release(text: &str) -> Result<Release, String> {
    let v: serde_json::Value = serde_json::from_str(text).map_err(|e| format!("Antwort unlesbar: {e}"))?;
    let s = |k: &str| v.get(k).and_then(|x| x.as_str()).unwrap_or("").to_string();
    let tag = s("tag_name");
    if tag.is_empty() {
        return Err("Antwort enthält keine Version.".into());
    }
    let assets = v
        .get("assets")
        .and_then(|a| a.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|a| {
                    Some(Asset {
                        name: a.get("name")?.as_str()?.to_string(),
                        url: a.get("browser_download_url")?.as_str()?.to_string(),
                        size: a.get("size").and_then(|x| x.as_u64()).unwrap_or(0),
                    })
                })
                .collect()
        })
        .unwrap_or_default();
    Ok(Release { tag, name: s("name"), body: s("body"), date: s("published_at").chars().take(10).collect(), assets })
}

/// Lesbare Größe
pub fn human(bytes: u64) -> String {
    if bytes >= 1 << 20 {
        format!("{:.1} MB", bytes as f64 / (1 << 20) as f64)
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}

pub fn load_installed(data: &Path) -> Option<Installed> {
    let text = fs::read_to_string(data.join("installed.json")).ok()?;
    let v: serde_json::Value = serde_json::from_str(&text).ok()?;
    let version = v.get("version")?.as_str()?.to_string();
    let dir = data.join("game").join(&version);
    let inst = Installed { version, dir };
    inst.exe().exists().then_some(inst)
}

fn download(url: &str, to: &Path, size_hint: u64, progress: &mut dyn FnMut(u64, u64)) -> Result<String, String> {
    let resp = agent().get(url).call().map_err(|e| format!("Download fehlgeschlagen: {e}"))?;
    let total = resp.header("Content-Length").and_then(|v| v.parse().ok()).unwrap_or(size_hint);
    let mut reader = resp.into_reader();
    let mut file = fs::File::create(to).map_err(|e| format!("Kann nicht speichern: {e}"))?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    let mut done = 0u64;
    loop {
        let n = reader.read(&mut buf).map_err(|e| format!("Download abgebrochen: {e}"))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n]).map_err(|e| format!("Kann nicht speichern: {e}"))?;
        hasher.update(&buf[..n]);
        done += n as u64;
        progress(done, total.max(done));
    }
    Ok(hex(&hasher.finalize()))
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// Erwarteten Hash aus SHA256SUMS.txt ("<hex>  <datei>")
pub fn expected_hash(sums: &str, file: &str) -> Option<String> {
    sums.lines().find_map(|l| {
        let mut parts = l.split_whitespace();
        let hash = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        (name == file).then(|| hash.to_lowercase())
    })
}

fn extract(zip_path: &Path, to: &Path) -> Result<(), String> {
    let file = fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("Paket kaputt: {e}"))?;
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(|e| format!("Paket kaputt: {e}"))?;
        // Schutz gegen Pfade wie ../../ (Zip-Slip)
        let Some(rel) = entry.enclosed_name() else { continue };
        let out = to.join(rel);
        if entry.is_dir() {
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = out.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut f = fs::File::create(&out).map_err(|e| e.to_string())?;
        std::io::copy(&mut entry, &mut f).map_err(|e| e.to_string())?;
        #[cfg(unix)]
        if let Some(mode) = entry.unix_mode() {
            use std::os::unix::fs::PermissionsExt;
            let _ = fs::set_permissions(&out, fs::Permissions::from_mode(mode & 0o777));
        }
    }
    Ok(())
}

/// Spielpaket des Releases herunterladen, prüfen und installieren
pub fn install(release: &Release, data: &Path, progress: &mut dyn FnMut(u64, u64)) -> Result<Installed, String> {
    let name = platform_asset();
    let asset = release
        .asset(&name)
        .ok_or_else(|| format!("Für dieses System gibt es im Release {} kein Paket ({name}).", release.tag))?;
    let sums = release.asset(CHECKSUMS).ok_or_else(|| format!("Im Release {} fehlen die Prüfsummen.", release.tag))?;
    let dl = data.join("download");
    fs::create_dir_all(&dl).map_err(|e| format!("Kann den Ordner {} nicht anlegen: {e}", dl.display()))?;
    let part = dl.join(format!("{name}.part"));
    let hash = download(&asset.url, &part, asset.size, progress)?;
    // Prüfsumme: schützt vor kaputten oder abgebrochenen Downloads
    let text = agent()
        .get(&sums.url)
        .call()
        .map_err(|e| format!("Prüfsummen nicht ladbar: {e}"))?
        .into_string()
        .map_err(|e| e.to_string())?;
    match expected_hash(&text, &name) {
        Some(want) if want == hash => {}
        Some(_) => {
            let _ = fs::remove_file(&part);
            return Err("Prüfsumme stimmt nicht – der Download ist beschädigt. Bitte nochmal versuchen.".into());
        }
        None => return Err(format!("{name} fehlt in {CHECKSUMS}.")),
    }
    let games = data.join("game");
    let tmp = games.join(format!("{}.tmp", release.tag));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    extract(&part, &tmp)?;
    let inst = Installed { version: release.tag.clone(), dir: tmp.clone() };
    let exe = inst.exe();
    if !exe.exists() {
        return Err("Im Paket fehlt das Spiel.".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&exe, fs::Permissions::from_mode(0o755));
    }
    let final_dir = games.join(&release.tag);
    let _ = fs::remove_dir_all(&final_dir);
    fs::rename(&tmp, &final_dir).map_err(|e| format!("Installation fehlgeschlagen: {e}"))?;
    fs::write(data.join("installed.json"), serde_json::json!({ "version": release.tag }).to_string())
        .map_err(|e| e.to_string())?;
    // Alte Versionen und den Download wegräumen
    if let Ok(entries) = fs::read_dir(&games) {
        for e in entries.flatten() {
            if e.file_name().to_string_lossy() != release.tag {
                let _ = fs::remove_dir_all(e.path());
            }
        }
    }
    let _ = fs::remove_file(&part);
    Ok(Installed { version: release.tag.clone(), dir: final_dir })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_github_release() {
        let json = r#"{"tag_name":"v2.0.0","name":"Panzer 2.0","body":"Neu: Rust","published_at":"2026-10-22T10:00:00Z",
            "assets":[{"name":"panzer-game-linux-x86_64.zip","browser_download_url":"https://x/y.zip","size":1234}]}"#;
        let r = parse_release(json).unwrap();
        assert_eq!(r.tag, "v2.0.0");
        assert_eq!(r.date, "2026-10-22");
        assert_eq!(r.asset("panzer-game-linux-x86_64.zip").unwrap().size, 1234);
    }

    #[test]
    fn finds_checksum() {
        let sums = "abc123  panzer-game-linux-x86_64.zip\nDEF456 *panzer-game-windows-x86_64.zip\n";
        assert_eq!(expected_hash(sums, "panzer-game-windows-x86_64.zip").as_deref(), Some("def456"));
        assert_eq!(expected_hash(sums, "fehlt.zip"), None);
    }

    #[test]
    fn refuses_release_without_checksums() {
        let name = platform_asset();
        let feed = format!(
            r#"{{"tag_name":"v9.0.0","assets":[{{"name":"{name}","browser_download_url":"http://127.0.0.1:9/x.zip","size":1}}]}}"#
        );
        let release = parse_release(&feed).unwrap();
        let dir = std::env::temp_dir().join(format!("panzer-launcher-nosums-{}", std::process::id()));
        let err = install(&release, &dir, &mut |_, _| {}).unwrap_err();
        assert!(err.contains("Prüfsummen"), "{err}");
        assert!(!dir.exists(), "darf ohne Prüfsummen nichts anlegen");
    }

    /// Mini-HTTP-Server für den Test: liefert Dateien aus einer Liste aus
    fn serve(files: Vec<(String, Vec<u8>)>) -> String {
        use std::io::{BufRead, BufReader};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        std::thread::spawn(move || {
            for stream in listener.incoming().flatten() {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                let path = line.split_whitespace().nth(1).unwrap_or("/").trim_start_matches('/').to_string();
                loop {
                    let mut h = String::new();
                    if reader.read_line(&mut h).unwrap() <= 2 {
                        break;
                    }
                }
                let mut out = stream;
                match files.iter().find(|f| f.0 == path) {
                    Some((_, body)) => {
                        let _ = write!(
                            out,
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = out.write_all(body);
                    }
                    None => {
                        let _ = write!(out, "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                    }
                }
            }
        });
        base
    }

    fn game_zip() -> Vec<u8> {
        let mut buf = std::io::Cursor::new(Vec::new());
        {
            let mut z = zip::ZipWriter::new(&mut buf);
            let opts = zip::write::SimpleFileOptions::default().unix_permissions(0o755);
            z.start_file(if cfg!(windows) { "panzer.exe" } else { "panzer" }, opts).unwrap();
            z.write_all(b"#!/bin/sh\necho spiel\n").unwrap();
            z.start_file("../boese.txt", opts).unwrap(); // Zip-Slip-Versuch
            z.write_all(b"x").unwrap();
            z.finish().unwrap();
        }
        buf.into_inner()
    }

    #[test]
    fn installs_and_rejects_tampered_package() {
        let name = platform_asset();
        let zip = game_zip();
        let good = hex(&Sha256::digest(&zip));
        let dir = std::env::temp_dir().join(format!("panzer-launcher-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        for (tag, sums) in
            [("v9.0.0", format!("{good}  {name}\n")), ("v9.0.1", format!("{}  {name}\n", "0".repeat(64)))]
        {
            let listener_files = vec![(name.clone(), zip.clone()), (CHECKSUMS.to_string(), sums.clone().into_bytes())];
            let base = serve(listener_files);
            let feed = format!(
                r#"{{"tag_name":"{tag}","name":"Test","body":"","published_at":"2026-10-22","assets":[
                {{"name":"{name}","browser_download_url":"{base}/{name}","size":{}}},
                {{"name":"{CHECKSUMS}","browser_download_url":"{base}/{CHECKSUMS}","size":1}}]}}"#,
                zip.len()
            );
            let release = parse_release(&feed).unwrap();
            let mut calls = 0;
            let result = install(&release, &dir, &mut |_, _| calls += 1);
            if tag == "v9.0.0" {
                let inst = result.expect("Installation sollte klappen");
                assert!(inst.exe().exists());
                assert!(calls > 0);
                assert_eq!(load_installed(&dir).unwrap().version, "v9.0.0");
                assert!(!dir.join("boese.txt").exists() && !dir.join("game/boese.txt").exists());
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    assert_eq!(fs::metadata(inst.exe()).unwrap().permissions().mode() & 0o111, 0o111);
                }
            } else {
                assert!(result.unwrap_err().contains("Prüfsumme"));
                // die alte Version bleibt installiert
                assert_eq!(load_installed(&dir).unwrap().version, "v9.0.0");
            }
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
