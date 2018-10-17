/// Generate the header
use colored::{Color, Colorize};
use rand::Rng;

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

/// Generate the `Deadmock` header.
crate fn header() {
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