use sysinfo::System;
use std::{fs, thread, time::Duration};

use sysinfo::MINIMUM_CPU_UPDATE_INTERVAL;

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

// Fallback when cpufreq sysfs is unavailable (e.g. some VMs): parse the
// per-core "cpu MHz" lines from /proc/cpuinfo, returned as kHz to match
// the sysfs units expected by format_freq.
fn cpuinfo_freqs() -> Vec<u64> {
    fs::read_to_string("/proc/cpuinfo")
        .map(|s| {
            s.lines()
                .filter(|l| l.starts_with("cpu MHz"))
                .filter_map(|l| l.split(':').nth(1)?.trim().parse::<f64>().ok())
                .map(|mhz| (mhz * 1000.0) as u64)
                .collect()
        })
        .unwrap_or_default()
}

fn read_meminfo(key: &str) -> u64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with(key))
                .and_then(|l| l.split_whitespace().nth(1)?.parse::<u64>().ok())
        })
        .map(|kb| kb * 1024)
        .unwrap_or(0)
}

fn render(sys: &System, cores: usize) {
    let total_ram = sys.total_memory();
    let used_ram = sys.used_memory();
    let avail_ram = sys.free_memory();
    let cache_ram = read_meminfo("Buffers:") + read_meminfo("Cached:") + read_meminfo("SReclaimable:");

    println!("┌─────────────────────────────────────┐");
    println!("│       System Resource Monitor       │");

    println!("├──────────────── CPU ────────────────┤");
    println!("│  {:<9} : {:<23}│", "Cores", cores);
    println!("├─────────────────────────────────────┤");
    let fallback_freqs = cpuinfo_freqs();
    for (i, cpu) in sys.cpus().iter().enumerate() {
        let pct = cpu.cpu_usage();
        let cur = match read_freq(i, "scaling_cur_freq") {
            0 => fallback_freqs.get(i).copied().unwrap_or(0),
            f => f,
        };
        let label = format!("Core {:>2}", i);
        println!("│  {:<9} : {} {}│", label, bar(pct, 9), format_freq(cur));
    }

    println!("├──────────────── RAM ────────────────┤");
    println!("│  {:<9} : {:<23}│", "Total", format_bytes(total_ram));
    println!("│  {:<9} : {:<23}│", "Used", format_bytes(used_ram));
    println!("│  {:<9} : {:<23}│", "Avail", format_bytes(avail_ram));
    println!("│  {:<9} : {:<23}│", "Cache", format_bytes(cache_ram));
    let ram_pct = used_ram as f32 / total_ram as f32 * 100.0;
    println!("│  {:<9} : {}│", "Usage", bar(ram_pct, 14));

    println!("└─────────────────────────────────────┘");
    print!("  Press Ctrl+C to exit");
}

fn main() {
    let mut sys = System::new_all();
    let cores = cpu_count();

    // Prime CPU usage: it's measured as a delta between two refreshes,
    // so an initial sample (with the required minimum interval) is needed
    // before the first reading is meaningful.
    sys.refresh_cpu_usage();
    thread::sleep(MINIMUM_CPU_UPDATE_INTERVAL);

    loop {
        sys.refresh_cpu_usage();
        sys.refresh_memory();
        print!("\x1b[2J\x1b[H");
        render(&sys, cores);
        thread::sleep(Duration::from_secs(1));
    }
}
