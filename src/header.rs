// Copyright (c) 2018 deadmock developers
//
// Licensed under the Apache License, Version 2.0
// <LICENSE-APACHE or http://www.apache.org/licenses/LICENSE-2.0> or the MIT
// license <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. All files in the project carrying such notice may not be copied,
// modified, or distributed except according to those terms.

//! `deadmock` header
use colored::{Color, Colorize};
use rand::Rng;
use slog::Logger;
use slog::{b, kv, log, record, record_static, trace};
use slog_try::try_trace;
use std::convert::TryFrom;
use tokio::net::TcpStream;

fn random_color() -> Color {
    let num = rand::thread_rng().gen_range(0, 7);

    match num {
        1 => Color::Green,
        2 => Color::Yellow,
        3 => Color::Blue,
        4 => Color::Magenta,
        5 => Color::Cyan,
        6 => Color::White,
        _ => Color::Red,
    }
}

pub fn header() {
    let color = random_color();
    println!("{}", "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2584}     \u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}    \u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2584}    \u{2584}\u{2584}\u{2584}\u{2584}\u{2588}\u{2588}\u{2588}\u{2584}\u{2584}\u{2584}\u{2584}    \u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2584}   \u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}    \u{2584}\u{2588}   \u{2584}\u{2588}\u{2584} ".color(color));
    println!("{}", "\u{2588}\u{2588}\u{2588}   \u{2580}\u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}   \u{2580}\u{2588}\u{2588}\u{2588} \u{2584}\u{2588}\u{2588}\u{2580}\u{2580}\u{2580}\u{2588}\u{2588}\u{2588}\u{2580}\u{2580}\u{2580}\u{2588}\u{2588}\u{2584} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588} \u{2584}\u{2588}\u{2588}\u{2588}\u{2580} ".color(color));
    println!("{}", "\u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}    \u{2588}\u{2580}    \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2580}    \u{2588}\u{2588}\u{2588}\u{2590}\u{2588}\u{2588}\u{2580}   ".color(color));
    println!("{}", "\u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588}  \u{2584}\u{2588}\u{2588}\u{2588}\u{2584}\u{2584}\u{2584}       \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}         \u{2584}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2580}    ".color(color));
    println!("{}", "\u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2580}\u{2580}\u{2588}\u{2588}\u{2588}\u{2580}\u{2580}\u{2580}     \u{2580}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}        \u{2580}\u{2580}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2584}    ".color(color));
    println!("{}", "\u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}    \u{2588}\u{2584}    \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2584}    \u{2588}\u{2588}\u{2588}\u{2590}\u{2588}\u{2588}\u{2584}   ".color(color));
    println!("{}", "\u{2588}\u{2588}\u{2588}   \u{2584}\u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}   \u{2584}\u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588} \u{2588}\u{2588}\u{2588}    \u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588} \u{2580}\u{2588}\u{2588}\u{2588}\u{2584} ".color(color));
    println!("{}", "\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2580}    \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}   \u{2588}\u{2588}\u{2588}    \u{2588}\u{2580}  \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2580}   \u{2580}\u{2588}   \u{2588}\u{2588}\u{2588}   \u{2588}\u{2580}   \u{2580}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2580}  \u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2588}\u{2580}    \u{2588}\u{2588}\u{2588}   \u{2580}\u{2588}\u{2580} ".color(color));
    println!();
    println!(
        "{}:    {}",
        "Build Version".bold(),
        env!("VERGEN_SEMVER").bold().green()
    );
    println!(
        "{}:  {}",
        "Build Timestamp".bold(),
        env!("VERGEN_BUILD_TIMESTAMP").bold().green()
    );
    println!(
        "{}:  {}",
        "Last Commit SHA".bold(),
        env!("VERGEN_SHA").bold().green()
    );
    println!(
        "{}: {}",
        "Last Commit Date".bold(),
        env!("VERGEN_COMMIT_DATE").bold().green()
    );
    println!();
}

#[allow(clippy::cast_precision_loss)]
fn as_mebibytes(val: usize) -> f64 {
    (val as f64) / 1_048_576.
}

pub fn socket_info(socket: &TcpStream, stdout: &Option<Logger>) {
    let local_addr = socket
        .local_addr()
        .ok()
        .map_or_else(|| "Unknown".to_string(), |v| v.to_string());
    let peer_addr = socket
        .peer_addr()
        .ok()
        .map_or_else(|| "Unknown".to_string(), |v| v.to_string());
    let tcp_nodelay = socket.nodelay().unwrap_or(false);
    let recv_size = socket.recv_buffer_size().map(as_mebibytes).unwrap_or(0.);
    let send_size = socket.send_buffer_size().map(as_mebibytes).unwrap_or(0.);
    let ttl = socket.ttl().unwrap_or(0);
    let linger =
        u64::try_from(socket.linger().unwrap_or(None).map_or(0, |v| v.as_millis())).unwrap_or(0);
    let keepalive = u64::try_from(
        socket
            .keepalive()
            .unwrap_or(None)
            .map_or(0, |v| v.as_millis()),
    ).unwrap_or(0);

    try_trace!(
        stdout,
        "Accepting connection";
        "SO_SNDBUF" => format!("{:.3} MiB", send_size),
        "SO_RCVBUF" => format!("{:.3} MiB", recv_size),
        "SO_LINGER" => linger,
        "SO_KEEPALIVE" => keepalive,
        "IP_TTL" => ttl,
        "TCP_NODELAY" => tcp_nodelay,
        "local_addr" => local_addr,
        "peer_addr" => peer_addr,
    );
}
