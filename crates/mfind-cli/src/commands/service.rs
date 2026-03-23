//! Service command implementation
//!
//! Background service management using launchd on macOS

use clap::Subcommand;
use console::style;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Background service management commands
#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Install background service
    Install,

    /// Start service
    Start,

    /// Stop service
    Stop,

    /// Uninstall service
    Uninstall,

    /// Show service status
    Status,

    /// Show service logs
    Logs(ServiceLogsCommand),
}

impl ServiceCommand {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            ServiceCommand::Install => self.install(),
            ServiceCommand::Start => self.start(),
            ServiceCommand::Stop => self.stop(),
            ServiceCommand::Uninstall => self.uninstall(),
            ServiceCommand::Status => self.status(),
            ServiceCommand::Logs(cmd) => cmd.run(),
        }
    }

    fn get_launchd_dir() -> anyhow::Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Home directory not found"))?;
        let launchd_dir = home.join("Library/LaunchAgents");
        fs::create_dir_all(&launchd_dir)?;
        Ok(launchd_dir)
    }

    fn get_plist_path() -> anyhow::Result<PathBuf> {
        Ok(Self::get_launchd_dir()?.join("com.mfind.daemon.plist"))
    }

    fn get_executable_path() -> anyhow::Result<PathBuf> {
        // Try to find mfind executable
        let exe = std::env::current_exe()?;
        Ok(exe)
    }

    fn generate_plist() -> anyhow::Result<String> {
        let exe = Self::get_executable_path()?;
        let exe_path = exe.display();

        Ok(format!(r##"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.mfind.daemon</string>

    <key>ProgramArguments</key>
    <array>
        <string>{0}</string>
        <string>serve</string>
    </array>

    <key>RunAtLoad</key>
    <true/>

    <key>KeepAlive</key>
    <true/>

    <key>WorkingDirectory</key>
    <string>/tmp</string>

    <key>StandardOutPath</key>
    <string>/tmp/mfind.out.log</string>

    <key>StandardErrorPath</key>
    <string>/tmp/mfind.err.log</string>

    <key>ProcessType</key>
    <string>Background</string>

    <key>LowPriorityIO</key>
    <true/>

    <key>Nice</key>
    <integer>10</integer>
</dict>
</plist>"##, exe_path))
    }

    fn install(&self) -> anyhow::Result<()> {
        println!(
            "{} Installing background service...",
            style("→").blue()
        );

        #[cfg(not(target_os = "macos"))]
        {
            eprintln!(
                "{} launchd is only available on macOS",
                style("⚠").yellow()
            );
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            // Generate plist
            let plist_content = Self::generate_plist()?;
            let plist_path = Self::get_plist_path()?;

            // Write plist file
            let mut file = fs::File::create(&plist_path)?;
            file.write_all(plist_content.as_bytes())?;

            println!(
                "{} Created launchd plist: {}",
                style("✓").green(),
                style(plist_path.display()).cyan()
            );

            // Load the service
            let output = Command::new("launchctl")
                .args(["load", "-w", plist_path.to_str().unwrap()])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        println!(
                            "{} Service installed and started",
                            style("✓").green()
                        );
                        println!();
                        println!("  Label:   {}", style("com.mfind.daemon").cyan());
                        println!("  Status:  {}", style("Running").green());
                        println!();
                        println!(
                            "{} Use {} to stop the service",
                            style("ℹ").blue(),
                            style("mfind service stop").cyan()
                        );
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        eprintln!(
                            "{} Failed to load service: {}",
                            style("✗").red(),
                            stderr
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to load service: {}",
                        style("✗").red(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    fn start(&self) -> anyhow::Result<()> {
        println!(
            "{} Starting service...",
            style("→").blue()
        );

        #[cfg(not(target_os = "macos"))]
        {
            eprintln!(
                "{} launchd is only available on macOS",
                style("⚠").yellow()
            );
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            let plist_path = Self::get_plist_path()?;

            if !plist_path.exists() {
                println!(
                    "{} Service not installed. Run {} first.",
                    style("⚠").yellow(),
                    style("mfind service install").cyan()
                );
                return Ok(());
            }

            let output = Command::new("launchctl")
                .args(["start", "com.mfind.daemon"])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        println!(
                            "{} Service started",
                            style("✓").green()
                        );
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        // Service might already be running, which is ok
                        if stderr.contains("already running") {
                            println!(
                                "{} Service already running",
                                style("ℹ").blue()
                            );
                        } else {
                            eprintln!(
                                "{} Failed to start service: {}",
                                style("✗").red(),
                                stderr
                            );
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to start service: {}",
                        style("✗").red(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    fn stop(&self) -> anyhow::Result<()> {
        println!(
            "{} Stopping service...",
            style("→").blue()
        );

        #[cfg(not(target_os = "macos"))]
        {
            eprintln!(
                "{} launchd is only available on macOS",
                style("⚠").yellow()
            );
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            let output = Command::new("launchctl")
                .args(["stop", "com.mfind.daemon"])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        println!(
                            "{} Service stopped",
                            style("✓").green()
                        );
                    } else {
                        let stderr = String::from_utf8_lossy(&out.stderr);
                        println!(
                            "{} Service status: {}",
                            style("ℹ").blue(),
                            stderr.trim()
                        );
                    }
                }
                Err(e) => {
                    eprintln!(
                        "{} Failed to stop service: {}",
                        style("✗").red(),
                        e
                    );
                }
            }
        }

        Ok(())
    }

    fn uninstall(&self) -> anyhow::Result<()> {
        println!(
            "{} Uninstalling service...",
            style("→").blue()
        );

        #[cfg(not(target_os = "macos"))]
        {
            eprintln!(
                "{} launchd is only available on macOS",
                style("⚠").yellow()
            );
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            // First stop the service
            let _ = Command::new("launchctl")
                .args(["stop", "com.mfind.daemon"])
                .output();

            // Unload the service
            let plist_path = Self::get_plist_path()?;
            if plist_path.exists() {
                let output = Command::new("launchctl")
                    .args(["unload", "-w", plist_path.to_str().unwrap()])
                    .output();

                if let Ok(out) = output {
                    if !out.status.success() {
                        eprintln!(
                            "{} Failed to unload service",
                            style("⚠").yellow()
                        );
                    }
                }

                // Remove plist file
                fs::remove_file(&plist_path)?;
                println!(
                    "{} Removed launchd plist",
                    style("✓").green()
                );
            }

            println!(
                "{} Service uninstalled",
                style("✓").green()
            );
        }

        Ok(())
    }

    fn status(&self) -> anyhow::Result<()> {
        println!("{}", style("Service Status").bold());
        println!();

        #[cfg(not(target_os = "macos"))]
        {
            println!("  Platform: {}", style("Not macOS (launchd not available)").yellow());
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            let plist_path = Self::get_plist_path()?;

            if !plist_path.exists() {
                println!("  Status:   {}", style("Not installed").yellow());
                println!();
                println!(
                    "{} Run {} to install the service.",
                    style("ℹ").blue(),
                    style("mfind service install").cyan()
                );
                return Ok(());
            }

            // Check if service is loaded
            let output = Command::new("launchctl")
                .arg("list")
                .output();

            let is_running = match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    stdout.contains("com.mfind.daemon")
                }
                Err(_) => false,
            };

            let status_str = if is_running {
                style("Running").green().to_string()
            } else {
                style("Stopped").yellow().to_string()
            };

            println!("  Label:    {}", style("com.mfind.daemon").cyan());
            println!("  Status:   {}", status_str);
            println!("  Plist:    {}", style(plist_path.display()).dim());

            if is_running {
                println!();
                println!(
                    "{} Use {} to stop the service",
                    style("ℹ").blue(),
                    style("mfind service stop").cyan()
                );
                println!(
                    "{} Use {} to uninstall",
                    style("ℹ").blue(),
                    style("mfind service uninstall").cyan()
                );
            }
        }

        Ok(())
    }
}

/// Service logs command
#[derive(clap::Args)]
pub struct ServiceLogsCommand {
    /// Number of lines to show
    #[arg(short = 'n', long, default_value = "50")]
    pub lines: usize,

    /// Follow log output
    #[arg(short = 'f', long)]
    pub follow: bool,
}

impl ServiceLogsCommand {
    pub fn run(&self) -> anyhow::Result<()> {
        #[cfg(not(target_os = "macos"))]
        {
            eprintln!(
                "{} Service logs are only available on macOS",
                style("⚠").yellow()
            );
            return Ok(());
        }

        #[cfg(target_os = "macos")]
        {
            let out_log = Path::new("/tmp/mfind.out.log");
            let err_log = Path::new("/tmp/mfind.err.log");

            if self.follow {
                println!("{} Following logs (Ctrl+C to stop)...", style("ℹ").blue());
                println!();

                // Use tail -f
                let mut cmd = Command::new("tail")
                    .arg("-f")
                    .arg("-n")
                    .arg(self.lines.to_string())
                    .arg(out_log)
                    .spawn()?;

                // Wait for Ctrl+C
                cmd.wait()?;
            } else {
                println!("{}", style("=== Output Log ===").bold());
                if out_log.exists() {
                    let content = fs::read_to_string(out_log)?;
                    println!("{}", content);
                } else {
                    println!("(no output log)");
                }

                println!();
                println!("{}", style("=== Error Log ===").bold());
                if err_log.exists() {
                    let content = fs::read_to_string(err_log)?;
                    println!("{}", content);
                } else {
                    println!("(no error log)");
                }
            }
        }

        #[cfg(not(target_os = "macos"))]
        {
            println!("{}", style("Service logs are not available on this platform").yellow());
        }

        Ok(())
    }
}
