use anyhow::{bail, Result};
use axum::http::HeaderMap;
use std::net::{IpAddr, SocketAddr};

/// Non-dry-run API actions require loopback origin or `X-LPMR-Token` matching `LPMR_EXECUTE_TOKEN`.
pub fn authorize_execute(peer: Option<SocketAddr>, headers: &HeaderMap) -> Result<()> {
    if let Ok(expected) = std::env::var("LPMR_EXECUTE_TOKEN") {
        let got = headers
            .get("x-lpmr-token")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        if got == expected {
            return Ok(());
        }
        bail!("missing or invalid X-LPMR-Token");
    }
    let allow_remote = std::env::var("LPMR_ALLOW_REMOTE_EXECUTE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);
    if allow_remote {
        return Ok(());
    }
    let Some(peer) = peer else {
        bail!("could not verify client address; bind to a known interface or set LPMR_EXECUTE_TOKEN");
    };
    if is_loopback(peer.ip()) {
        return Ok(());
    }
    bail!("real execution blocked from non-loopback clients; use localhost, set LPMR_EXECUTE_TOKEN, or LPMR_ALLOW_REMOTE_EXECUTE=1");
}

fn is_loopback(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback(),
        IpAddr::V6(v6) => v6.is_loopback(),
    }
}
