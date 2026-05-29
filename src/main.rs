use sysinfo::System;
use std::{fs, thread, time::Duration};

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

fn bar(pct: f32, width: usize) -> String {
    let c = color(pct);
    let filled = ((pct / 100.0) * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    format!("[{}{}{}{}{}{}] {:5.1}%",
        c, "#".repeat(filled), RESET,
        "\x1b[90m", "-".repeat(empty), RESET,
        pct
    )
}

fn cpu_count() -> usize {
    fs::read_to_string("/proc/cpuinfo")
        .map(|s| s.lines().filter(|l| l.starts_with("processor")).count())
        .unwrap_or(0)
}

fn format_freq(khz: u64) -> String {
    let mhz = khz / 1000;
    if mhz >= 1000 {
        format!("{:.1}G", mhz as f64 / 1000.0)
    } else {
        format!("{:>3}M", mhz)
    }
}

fn read_freq(core: usize, file: &str) -> u64 {
    fs::read_to_string(format!("/sys/devices/system/cpu/cpu{}/cpufreq/{}", core, file))
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn render(sys: &System, cores: usize) {
    let total_ram = sys.total_memory();
    let used_ram = sys.used_memory();
    let free_ram = sys.free_memory();

    println!("┌─────────────────────────────────────┐");
    println!("│       System Resource Monitor       │");

    println!("├──────────────── CPU ────────────────┤");
    println!("│  {:<9} : {:<23}│", "Cores", cores);
    println!("├─────────────────────────────────────┤");
    for i in 0..cores {
        let cur = read_freq(i, "scaling_cur_freq");
        let max = read_freq(i, "cpuinfo_max_freq");
        let pct = if max > 0 { cur as f32 / max as f32 * 100.0 } else { 0.0 };
        let label = format!("Core {:>2}", i);
        println!("│  {:<9} : {} {}│", label, bar(pct, 9), format_freq(cur));
    }

    println!("├──────────────── RAM ────────────────┤");
    println!("│  {:<9} : {:<23}│", "Total", format_bytes(total_ram));
    println!("│  {:<9} : {:<23}│", "Used", format_bytes(used_ram));
    println!("│  {:<9} : {:<23}│", "Free", format_bytes(free_ram));
    let ram_pct = used_ram as f32 / total_ram as f32 * 100.0;
    println!("│  {:<9} : {}│", "Usage", bar(ram_pct, 14));

    println!("└─────────────────────────────────────┘");
    print!("  Press Ctrl+C to exit");
}

fn main() {
    let mut sys = System::new_all();
    let cores = cpu_count();

    loop {
        sys.refresh_memory();
        print!("\x1b[2J\x1b[H");
        render(&sys, cores);
        thread::sleep(Duration::from_secs(1));
    }
}
