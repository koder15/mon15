use sysinfo::System;
use std::{thread, time::Duration};

const RESET: &str = "\x1b[0m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";

fn color(pct: f32) -> &'static str {
    if pct < 50.0 { GREEN }
    else if pct < 80.0 { YELLOW }
    else { RED }
}

fn format_bytes(bytes: u64) -> String {
    const GB: u64 = 1024 * 1024 * 1024;
    const MB: u64 = 1024 * 1024;
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    }
}

// Always 23 visual chars: [ + 14 + ] + space + {:5.1}%
fn cpu_bar(pct: f32, width: usize) -> String {
    let c = color(pct);
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}{}{}{}{}] {:5.1}%",
        c, "#".repeat(filled), RESET,
        "\x1b[90m", "-".repeat(empty), RESET,
        pct
    )
}

fn ram_bar(used: u64, total: u64, width: usize) -> String {
    let pct = (used as f64 / total as f64 * 100.0) as f32;
    let c = color(pct);
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}{}{}{}{}] {:5.1}%",
        c, "#".repeat(filled), RESET,
        "\x1b[90m", "-".repeat(empty), RESET,
        pct
    )
}

fn render(sys: &System) {
    let cpus = sys.cpus();
    let total_ram = sys.total_memory();
    let used_ram = sys.used_memory();
    let free_ram = sys.free_memory();

    // Box inner width = 37, total = 39
    // Bar lines use {} (not {:<23}) since bar is always exactly 23 visual chars
    println!("┌─────────────────────────────────────┐");
    println!("│       System Resource Monitor       │");

    println!("├──────────────── CPU ────────────────┤");
    println!("│  {:<9} : {:<23}│", "Cores", cpus.len());
    println!("├─────────────────────────────────────┤");
    for (i, cpu) in cpus.iter().enumerate() {
        let label = format!("Core {:>2}", i);
        println!("│  {:<9} : {}│", label, cpu_bar(cpu.cpu_usage(), 14));
    }

    println!("├──────────────── RAM ────────────────┤");
    println!("│  {:<9} : {:<23}│", "Total", format_bytes(total_ram));
    println!("│  {:<9} : {:<23}│", "Used", format_bytes(used_ram));
    println!("│  {:<9} : {:<23}│", "Free", format_bytes(free_ram));
    println!("│  {:<9} : {}│", "Usage", ram_bar(used_ram, total_ram, 14));

    println!("└─────────────────────────────────────┘");
    print!("  Press Ctrl+C to exit");
}

fn main() {
    let mut sys = System::new_all();

    sys.refresh_cpu_usage();
    thread::sleep(Duration::from_millis(500));

    loop {
        sys.refresh_all();
        print!("\x1b[2J\x1b[H");
        render(&sys);
        thread::sleep(Duration::from_secs(1));
    }
}
