use chrono::Utc;
use colored::*;

#[allow(dead_code)]
pub struct Logger {}

#[derive(Debug)]
enum LogPriority {
    #[allow(dead_code)]
    Emerg,
    #[allow(dead_code)]
    Alert,
    #[allow(dead_code)]
    Critical,
    #[allow(dead_code)]
    Error,
    #[allow(dead_code)]
    Warning,
    #[allow(dead_code)]
    Notice,
    #[allow(dead_code)]
    Info,
    #[allow(dead_code)]
    Debug,
}

impl Logger {
    #[allow(dead_code)]
    pub fn new() -> Logger {
        Logger {}
    }

    fn logging(&self, priority: &LogPriority, message: &str) {
        let now = Utc::now().to_rfc3339();

        match priority {
            LogPriority::Emerg => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).on_bright_red().bold(),
                    message
                )
            }
            LogPriority::Alert => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).on_bright_red().bold(),
                    message
                )
            }
            LogPriority::Critical => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).on_bright_red().bold(),
                    message
                )
            }
            LogPriority::Error => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).bright_red().bold(),
                    message
                )
            }
            LogPriority::Warning => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).bright_yellow().bold(),
                    message
                )
            }
            LogPriority::Notice => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).bright_cyan().bold(),
                    message
                )
            }
            LogPriority::Info => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).bright_green().bold(),
                    message
                )
            }
            LogPriority::Debug => {
                println!(
                    "{} {}: {}",
                    now,
                    format!("{:?}", priority).bright_white().bold(),
                    message
                )
            }
        }
    }

    // POSIX: 0 - Emergency
    #[allow(dead_code)]
    pub fn emerg(&self, message: &str) {
        self.logging(&LogPriority::Emerg, message)
    }

    /** @deprecated use `emerg(message:)` instead. */
    #[allow(dead_code)]
    pub fn panic(&self, message: &str) {
        self.logging(&LogPriority::Emerg, message)
    }

    // POSIX: 1 - Alert
    #[allow(dead_code)]
    pub fn alert(&self, message: &str) {
        self.logging(&LogPriority::Alert, message)
    }

    // POSIX: 2 - Critical
    #[allow(dead_code)]
    pub fn crit(&self, message: &str) {
        self.logging(&LogPriority::Critical, message)
    }

    // POSIX: 3 - Error
    #[allow(dead_code)]
    pub fn err(&self, message: &str) {
        self.logging(&LogPriority::Error, message)
    }

    /** @deprecated use `err(message:)` instead. */
    #[allow(dead_code)]
    pub fn error(&self, message: &str) {
        self.logging(&LogPriority::Error, message)
    }

    // POSIX: 4 - Warning
    #[allow(dead_code)]
    pub fn warning(&self, message: &str) {
        self.logging(&LogPriority::Warning, message)
    }

    /** @deprecated use `warning(message:)` instead. */
    #[allow(dead_code)]
    pub fn warn(&self, message: &str) {
        self.logging(&LogPriority::Warning, message)
    }

    // POSIX: 5 - Notice
    #[allow(dead_code)]
    pub fn notice(&self, message: &str) {
        self.logging(&LogPriority::Notice, message)
    }

    // POSIX: 6 - Informational
    #[allow(dead_code)]
    pub fn info(&self, message: &str) {
        self.logging(&LogPriority::Info, message)
    }

    // POSIX: 7 - Debug
    #[allow(dead_code)]
    pub fn debug(&self, message: &str) {
        self.logging(&LogPriority::Debug, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn message() -> String {
        String::from("Hello, NodeX!")
    }

    #[test]
    fn test_emerg() {
        let logger = Logger::new();

        logger.emerg(&message());
    }

    #[test]
    fn test_panic() {
        let logger = Logger::new();

        logger.panic(&message());
    }

    #[test]
    fn test_alert() {
        let logger = Logger::new();

        logger.alert(&message());
    }

    #[test]
    fn test_crit() {
        let logger = Logger::new();

        logger.crit(&message());
    }

    #[test]
    fn test_err() {
        let logger = Logger::new();

        logger.err(&message());
    }

    #[test]
    fn test_error() {
        let logger = Logger::new();

        logger.error(&message());
    }

    #[test]
    fn test_warning() {
        let logger = Logger::new();

        logger.warning(&message());
    }

    #[test]
    fn test_warn() {
        let logger = Logger::new();

        logger.warn(&message());
    }

    #[test]
    fn test_notice() {
        let logger = Logger::new();

        logger.notice(&message());
    }

    #[test]
    fn test_info() {
        let logger = Logger::new();

        logger.info(&message());
    }

    #[test]
    fn test_debug() {
        let logger = Logger::new();

        logger.debug(&message());
    }
}
