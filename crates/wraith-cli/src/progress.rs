//! Transfer progress display with progress bars.

use indicatif::{ProgressBar, ProgressStyle};
use std::time::Duration;

/// Transfer progress tracker
pub struct TransferProgress {
    bar: ProgressBar,
}

impl TransferProgress {
    /// Create a new progress tracker
    #[must_use]
    pub fn new(total_bytes: u64, filename: &str) -> Self {
        let bar = ProgressBar::new(total_bytes);

        bar.set_style(
            ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
                .expect("Invalid progress bar template")
                .progress_chars("#>-")
        );

        bar.set_message(format!("Transferring: {filename}"));

        Self { bar }
    }

    /// Update progress
    #[allow(dead_code)]
    pub fn update(&self, transferred_bytes: u64) {
        self.bar.set_position(transferred_bytes);
    }

    /// Set custom message
    #[allow(dead_code)]
    pub fn set_message(&self, msg: String) {
        self.bar.set_message(msg);
    }

    /// Finish with success message
    #[allow(dead_code)]
    pub fn finish(&self) {
        self.bar.finish_with_message("Transfer complete!");
    }

    /// Finish with custom message
    pub fn finish_with_message(&self, msg: String) {
        self.bar.finish_with_message(msg);
    }

    /// Abandon the progress bar (for errors)
    #[allow(dead_code)]
    pub fn abandon(&self) {
        self.bar.abandon();
    }
}

/// Format bytes in human-readable format
///
/// # Example
///
/// ```
/// use wraith_cli::progress::format_bytes;
///
/// assert_eq!(format_bytes(1024), "1.00 KB");
/// assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
/// assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
/// ```
#[must_use]
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_idx = 0;

    while size >= 1024.0 && unit_idx < UNITS.len() - 1 {
        size /= 1024.0;
        unit_idx += 1;
    }

    format!("{size:.2} {}", UNITS[unit_idx])
}

/// Format speed in human-readable format (bytes/sec)
///
/// # Example
///
/// ```
/// use wraith_cli::progress::format_speed;
///
/// assert_eq!(format_speed(1024.0), "1.00 KB/s");
/// assert_eq!(format_speed(1024.0 * 1024.0), "1.00 MB/s");
/// ```
#[must_use]
#[allow(dead_code)]
pub fn format_speed(bytes_per_sec: f64) -> String {
    format!("{}/s", format_bytes(bytes_per_sec as u64))
}

/// Format duration in human-readable format
///
/// # Example
///
/// ```
/// use wraith_cli::progress::format_duration;
/// use std::time::Duration;
///
/// assert_eq!(format_duration(Duration::from_secs(30)), "30s");
/// assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
/// assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
/// ```
#[must_use]
#[allow(dead_code)]
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        format!("{secs}s")
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    }
}

/// Format ETA from seconds
///
/// # Example
///
/// ```
/// use wraith_cli::progress::format_eta;
///
/// assert_eq!(format_eta(30.0), "30s");
/// assert_eq!(format_eta(90.5), "1m 30s");
/// ```
#[must_use]
#[allow(dead_code)]
pub fn format_eta(seconds: f64) -> String {
    format_duration(Duration::from_secs_f64(seconds))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0.00 B");
        assert_eq!(format_bytes(512), "512.00 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
        assert_eq!(format_bytes(1024_u64.pow(4)), "1.00 TB");
    }

    #[test]
    fn test_format_speed() {
        assert_eq!(format_speed(1024.0), "1.00 KB/s");
        assert_eq!(format_speed(1_048_576.0), "1.00 MB/s");
    }

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(Duration::from_secs(0)), "0s");
        assert_eq!(format_duration(Duration::from_secs(30)), "30s");
        assert_eq!(format_duration(Duration::from_secs(60)), "1m 0s");
        assert_eq!(format_duration(Duration::from_secs(90)), "1m 30s");
        assert_eq!(format_duration(Duration::from_secs(3600)), "1h 0m");
        assert_eq!(format_duration(Duration::from_secs(3661)), "1h 1m");
    }

    #[test]
    fn test_format_eta() {
        assert_eq!(format_eta(30.0), "30s");
        assert_eq!(format_eta(90.5), "1m 30s");
        assert_eq!(format_eta(3661.9), "1h 1m");
    }
}
