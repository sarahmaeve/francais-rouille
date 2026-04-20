//! Local static-file dev server.
//!
//! A minimal server for previewing `site/` during development without
//! depending on a Python sidecar or relying on `file://` URLs (which
//! break quiz / flashcard / crossword JS in modern browsers because of
//! relative-path and origin restrictions).
//!
//! Serves files with sensible MIME types, falls back to `index.html`
//! for directory requests, and 404s on anything else. The
//! Cloudflare-Pages `site/_headers` file is ignored — CSP is a
//! deployment concern, not a local-dev concern, and enforcing it
//! locally would get in the way of iteration. Use
//! `cargo run -- check-csp` after building to verify CSP compliance.
//!
//! Blocks the calling thread; handle Ctrl-C via the shell.
//!
//! This module was backported from the `intreccio` project, which
//! introduced the same idea first.

use std::path::{Path, PathBuf};

use tiny_http::{Header, Response, Server, StatusCode};

#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    #[error("failed to bind to {addr}: {source}")]
    Bind {
        addr: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Serve `site_dir` on `127.0.0.1:<port>` until the process is killed.
pub fn serve(site_dir: &Path, port: u16) -> Result<(), ServeError> {
    let site_dir = site_dir
        .canonicalize()
        .map_err(ServeError::Io)?;

    let addr = format!("127.0.0.1:{port}");
    let server = Server::http(&addr).map_err(|e| ServeError::Bind {
        addr: addr.clone(),
        source: e,
    })?;

    println!(
        "francais-rouille: serving {} on http://{addr}",
        site_dir.display(),
    );
    println!("  press Ctrl-C to stop");

    for request in server.incoming_requests() {
        let url = request.url().split('?').next().unwrap_or("/").to_string();
        let status = match handle(&site_dir, &url, request) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  500 {url} — {e}");
                continue;
            }
        };
        println!("  {} {}", status.0, url);
    }

    Ok(())
}

struct LoggedStatus(u16);

fn handle(
    site_dir: &Path,
    url: &str,
    request: tiny_http::Request,
) -> Result<LoggedStatus, std::io::Error> {
    // Resolve URL → filesystem path.
    let resolved = match resolve_path(site_dir, url) {
        Some(path) => path,
        None => {
            respond_not_found(request)?;
            return Ok(LoggedStatus(404));
        }
    };

    // Reject attempts to escape the site root via `..`. `resolve_path`
    // canonicalizes before returning, so a symlink or `..` trick that
    // points outside `site_dir` will fail this check.
    if !resolved.starts_with(site_dir) {
        respond_not_found(request)?;
        return Ok(LoggedStatus(404));
    }

    if !resolved.is_file() {
        respond_not_found(request)?;
        return Ok(LoggedStatus(404));
    }

    let content_type = mime_for(&resolved);
    let file = std::fs::File::open(&resolved)?;
    let response = Response::from_file(file).with_header(
        Header::from_bytes(&b"Content-Type"[..], content_type.as_bytes())
            .expect("valid header"),
    );
    request.respond(response)?;
    Ok(LoggedStatus(200))
}

/// Resolve a URL path to a filesystem path under `site_dir`.
///
/// - `/` → `site_dir/index.html`
/// - `/foo/` → `site_dir/foo/index.html`
/// - `/foo/bar.html` → `site_dir/foo/bar.html`
/// - URLs containing `..` segments that escape `site_dir` are rejected
///   at the caller (via the `starts_with(site_dir)` check on the
///   canonicalized path).
fn resolve_path(site_dir: &Path, url: &str) -> Option<PathBuf> {
    let trimmed = url.trim_start_matches('/');
    let relative = if trimmed.is_empty() || trimmed.ends_with('/') {
        format!("{trimmed}index.html")
    } else {
        trimmed.to_string()
    };

    let candidate = site_dir.join(&relative);
    // If the URL points to a directory, try `<dir>/index.html`.
    if candidate.is_dir() {
        let idx = candidate.join("index.html");
        return idx.canonicalize().ok();
    }
    candidate.canonicalize().ok()
}

fn respond_not_found(request: tiny_http::Request) -> std::io::Result<()> {
    let body = "404 Not Found";
    let response = Response::from_string(body)
        .with_status_code(StatusCode(404))
        .with_header(
            Header::from_bytes(&b"Content-Type"[..], &b"text/plain; charset=utf-8"[..])
                .expect("valid header"),
        );
    request.respond(response)
}

/// Guess the Content-Type of a file from its extension.
fn mime_for(path: &Path) -> String {
    mime_guess::from_path(path)
        .first_or_octet_stream()
        .to_string()
}

/// Parse `--port N` from CLI args, defaulting to `8000`.
pub fn parse_port(args: &[String]) -> Result<u16, Box<dyn std::error::Error>> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--port" {
            let value = args
                .get(i + 1)
                .ok_or("--port requires a value")?;
            return value
                .parse::<u16>()
                .map_err(|e| format!("--port: {e}").into());
        }
    }
    Ok(8000)
}

/// CLI entry point for the `serve` subcommand.
pub fn run_serve(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut site_dir = PathBuf::from("site");

    // Parse `--site DIR` and `--port N` in any order; ignore the first
    // two positional args (the program name and the subcommand).
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--site" {
            i += 1;
            site_dir = PathBuf::from(
                args.get(i)
                    .ok_or("--site requires a directory")?,
            );
        } else if args[i] == "--port" {
            // Handled by parse_port below; skip its value here.
            i += 1;
        } else if args[i] == "--help" || args[i] == "-h" {
            print_usage();
            return Ok(());
        }
        i += 1;
    }

    if !site_dir.exists() {
        return Err(format!(
            "site directory does not exist: {}\n\
             Run `cargo run -- build` first, or pass --site DIR.",
            site_dir.display(),
        )
        .into());
    }

    let port = parse_port(args)?;
    serve(&site_dir, port)?;
    Ok(())
}

fn print_usage() {
    println!("Usage: francais-rouille serve [--site DIR] [--port N]");
    println!();
    println!("Serves a static directory on 127.0.0.1 for local preview.");
    println!();
    println!("OPTIONS:");
    println!("  --site DIR    Directory to serve (default: site/)");
    println!("  --port N      TCP port to bind (default: 8000)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_slash_maps_to_index_html() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("index.html"), "<html></html>").unwrap();
        let root = dir.path().canonicalize().unwrap();
        let resolved = resolve_path(&root, "/").unwrap();
        assert_eq!(resolved, root.join("index.html"));
    }

    #[test]
    fn resolve_trailing_slash_maps_to_index_html() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("foo");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("index.html"), "<html></html>").unwrap();
        let root = dir.path().canonicalize().unwrap();
        let resolved = resolve_path(&root, "/foo/").unwrap();
        assert_eq!(resolved, root.join("foo").join("index.html"));
    }

    #[test]
    fn resolve_file_path() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("page.html"), "<html></html>").unwrap();
        let root = dir.path().canonicalize().unwrap();
        let resolved = resolve_path(&root, "/page.html").unwrap();
        assert_eq!(resolved, root.join("page.html"));
    }

    #[test]
    fn resolve_nested_file_path() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("chapters").join("b1-travail");
        std::fs::create_dir_all(&sub).unwrap();
        std::fs::write(sub.join("01_accueil_rh.html"), "<html></html>").unwrap();
        let root = dir.path().canonicalize().unwrap();
        let resolved = resolve_path(&root, "/chapters/b1-travail/01_accueil_rh.html").unwrap();
        assert_eq!(
            resolved,
            root.join("chapters").join("b1-travail").join("01_accueil_rh.html"),
        );
    }

    #[test]
    fn resolve_missing_file_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        assert!(resolve_path(&root, "/does-not-exist.html").is_none());
    }

    #[test]
    fn resolve_directory_without_index_returns_none() {
        let dir = tempfile::tempdir().unwrap();
        let sub = dir.path().join("empty");
        std::fs::create_dir_all(&sub).unwrap();
        let root = dir.path().canonicalize().unwrap();
        assert!(resolve_path(&root, "/empty/").is_none());
    }

    #[test]
    fn mime_for_known_extensions() {
        // HTML, CSS, and JS are what the dev server almost always serves;
        // MP3 matters because we'll be serving generated dialog audio; WOFF2
        // matters because `site/shared/fonts/` is loaded cross-page.
        assert_eq!(mime_for(Path::new("x.html")), "text/html");
        assert_eq!(mime_for(Path::new("x.css")), "text/css");
        assert_eq!(mime_for(Path::new("x.js")), "text/javascript");
        assert_eq!(mime_for(Path::new("x.mp3")), "audio/mpeg");
        assert_eq!(mime_for(Path::new("x.woff2")), "font/woff2");
        assert_eq!(mime_for(Path::new("x.webp")), "image/webp");
    }

    #[test]
    fn mime_for_unknown_extension_is_octet_stream() {
        assert_eq!(mime_for(Path::new("x.unknownext")), "application/octet-stream");
    }

    #[test]
    fn traversal_attempt_resolves_outside_root_to_none() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().canonicalize().unwrap();
        // `..` would escape `root` but there's no such file to canonicalize.
        assert!(resolve_path(&root, "/../outside").is_none());
    }

    #[test]
    fn parse_port_defaults_to_8000() {
        let args = vec!["bin".into(), "serve".into()];
        assert_eq!(parse_port(&args).unwrap(), 8000);
    }

    #[test]
    fn parse_port_reads_explicit_value() {
        let args = vec!["bin".into(), "serve".into(), "--port".into(), "3000".into()];
        assert_eq!(parse_port(&args).unwrap(), 3000);
    }

    #[test]
    fn parse_port_rejects_non_numeric() {
        let args = vec!["bin".into(), "serve".into(), "--port".into(), "abc".into()];
        assert!(parse_port(&args).is_err());
    }

    #[test]
    fn parse_port_rejects_missing_value() {
        let args = vec!["bin".into(), "serve".into(), "--port".into()];
        assert!(parse_port(&args).is_err());
    }

    #[test]
    fn parse_port_rejects_out_of_range() {
        // 65536 is outside u16 range and should fail to parse.
        let args = vec!["bin".into(), "serve".into(), "--port".into(), "65536".into()];
        assert!(parse_port(&args).is_err());
    }
}
