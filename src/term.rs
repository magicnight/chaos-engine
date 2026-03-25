// Terminal color helpers (ANSI escape codes)

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const DIM: &str = "\x1b[2m";

pub const RED: &str = "\x1b[31m";
pub const GREEN: &str = "\x1b[32m";
pub const YELLOW: &str = "\x1b[33m";
pub const BLUE: &str = "\x1b[34m";
pub const MAGENTA: &str = "\x1b[35m";
pub const CYAN: &str = "\x1b[36m";
pub const WHITE: &str = "\x1b[37m";

pub const BG_RED: &str = "\x1b[41m";
pub const BG_GREEN: &str = "\x1b[42m";
pub const BG_YELLOW: &str = "\x1b[43m";
pub const BG_CYAN: &str = "\x1b[46m";

/// Print a box header line (top border).
pub fn box_top(width: usize) {
    eprintln!(
        "{BOLD}{CYAN}\u{2554}{}{RESET}",
        "\u{2550}".repeat(width)
            + &format!("{BOLD}{CYAN}\u{2557}{RESET}")
    );
}

/// Print a box separator line.
pub fn box_sep(width: usize) {
    eprintln!(
        "{BOLD}{CYAN}\u{2560}{}{RESET}",
        "\u{2550}".repeat(width)
            + &format!("{BOLD}{CYAN}\u{2563}{RESET}")
    );
}

/// Print a box bottom line.
pub fn box_bottom(width: usize) {
    eprintln!(
        "{BOLD}{CYAN}\u{255a}{}{RESET}",
        "\u{2550}".repeat(width)
            + &format!("{BOLD}{CYAN}\u{255d}{RESET}")
    );
}

/// Print a box content line, padded to width.
pub fn box_line(content: &str, width: usize) {
    // Strip ANSI codes for length calculation
    let visible_len = strip_ansi_len(content);
    let padding = if visible_len < width {
        width - visible_len
    } else {
        0
    };
    eprintln!(
        "{BOLD}{CYAN}\u{2551}{RESET}  {content}{}{BOLD}{CYAN}\u{2551}{RESET}",
        " ".repeat(if padding > 2 { padding - 2 } else { 0 })
    );
}

/// Print an empty box line.
pub fn box_empty(width: usize) {
    eprintln!(
        "{BOLD}{CYAN}\u{2551}{}{BOLD}{CYAN}\u{2551}{RESET}",
        " ".repeat(width)
    );
}

/// Count visible characters (strip ANSI escape sequences).
pub fn strip_ansi_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for c in s.chars() {
        if in_escape {
            if c.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if c == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

/// Print a plain header bar (no box).
pub fn header(text: &str) {
    eprintln!();
    eprintln!("{BOLD}{CYAN}{}{RESET}", "\u{2550}".repeat(50));
    eprintln!("{BOLD}{CYAN}  {text}{RESET}");
    eprintln!("{BOLD}{CYAN}{}{RESET}", "\u{2550}".repeat(50));
}

/// Color a value green if positive, red if negative.
pub fn color_change(val: f64) -> (&'static str, &'static str) {
    if val >= 0.0 {
        (GREEN, RESET)
    } else {
        (RED, RESET)
    }
}

/// Tier color: T1=cyan, T2=blue, T3=yellow, T4=magenta, T5=green
pub fn tier_color(tier: u64) -> &'static str {
    match tier {
        1 => CYAN,
        2 => BLUE,
        3 => YELLOW,
        4 => MAGENTA,
        5 => GREEN,
        _ => WHITE,
    }
}

/// Severity color
pub fn severity_color(severity: &str) -> &'static str {
    match severity.to_uppercase().as_str() {
        "CRITICAL" => RED,
        "HIGH" => YELLOW,
        "MODERATE" | "MEDIUM" => YELLOW,
        "LOW" => GREEN,
        _ => WHITE,
    }
}

/// Progress bar: filled/total out of bar_width characters
pub fn progress_bar(ok: usize, total: usize, bar_width: usize) -> String {
    if total == 0 {
        return " ".repeat(bar_width);
    }
    let filled = (ok * bar_width) / total;
    let empty = bar_width - filled;
    format!(
        "{GREEN}{}{RESET}{DIM}{}{RESET}",
        "\u{2588}".repeat(filled),
        "\u{2591}".repeat(empty),
    )
}
