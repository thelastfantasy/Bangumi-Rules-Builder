use colored::*;
use log::{Level, LevelFilter, Log, Metadata, Record};
use std::io::{self, Write};

/// 自定义日志记录器，支持彩色输出和本地时间戳
pub struct CustomLogger {
    level: LevelFilter,
}

impl CustomLogger {
    pub fn new(level: LevelFilter) -> Self {
        Self { level }
    }

    /// 获取当前本地时间
    fn current_local_time() -> String {
        let now = chrono::Local::now();
        now.format("%Y-%m-%d %H:%M:%S%.3f").to_string()
    }

    /// 格式化日志级别
    fn format_level(level: Level) -> ColoredString {
        match level {
            Level::Error => "ERROR".red().bold(),
            Level::Warn => "WARN ".yellow().bold(),
            Level::Info => "INFO ".green().bold(),
            Level::Debug => "DEBUG".blue().bold(),
            Level::Trace => "TRACE".magenta().bold(),
        }
    }

    /// 格式化日志记录
    fn format_record(&self, record: &Record) -> String {
        let timestamp = Self::current_local_time();
        let level = Self::format_level(record.level());
        let target = record.target();
        let message = record.args();

        format!(
            "{} {} [{}] {}",
            timestamp.dimmed(),
            level,
            target.cyan(),
            message
        )
    }
}

impl Log for CustomLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            // 使用 stderr 输出错误和警告，避免干扰进度条
            let output = self.format_record(record);
            match record.level() {
                Level::Error | Level::Warn => {
                    eprintln!("{}", output);
                }
                _ => {
                    println!("{}", output);
                }
            }
        }
    }

    fn flush(&self) {
        // 确保所有输出都被刷新
        io::stdout().flush().ok();
        io::stderr().flush().ok();
    }
}

/// 初始化日志系统
pub fn init_logger(level: LevelFilter) -> Result<(), Box<dyn std::error::Error>> {
    let logger = CustomLogger::new(level);

    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(level);

    Ok(())
}

/// 设置默认日志级别
pub fn init_default() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量获取日志级别，默认为 info
    let level = match std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()).to_lowercase().as_str() {
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    };

    init_logger(level)
}