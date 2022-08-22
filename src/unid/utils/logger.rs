use chrono::{Utc, Local, DateTime, Date, format::format};
use colored::*;

pub struct Logger {
}

#[derive(Debug)]
enum LogPriority {
    EMERG,
    ALERT,
    CRITICAL,
    ERROR,
    WARNING,
    NOTICE,
    INFO,
    DEBUG,
}

impl Logger {
    pub fn new() -> Logger {
        Logger { }
    }

    pub fn logging(&self, priority: &LogPriority, message: &str) {
        let now = Utc::now().to_rfc3339();

        match priority {
            LogPriority::EMERG => {
                println!("{} {}: {}", now, format!("{:?}", priority).on_bright_red().bold(), message)
            },
            LogPriority::ALERT => {
                println!("{} {}: {}", now, format!("{:?}", priority).on_bright_red().bold(), message)
            },
            LogPriority::CRITICAL => {
                println!("{} {}: {}", now, format!("{:?}", priority).on_bright_red().bold(), message)
            },
            LogPriority::ERROR => {
                println!("{} {}: {}", now, format!("{:?}", priority).bright_red().bold(), message)
            },
            LogPriority::WARNING => {
                println!("{} {}: {}", now, format!("{:?}", priority).bright_yellow().bold(), message)
            },
            LogPriority::NOTICE => {
                println!("{} {}: {}", now, format!("{:?}", priority).bright_cyan().bold(), message)
            },
            LogPriority::INFO => {
                println!("{} {}: {}", now, format!("{:?}", priority).bright_green().bold(), message)
            },
            LogPriority::DEBUG => {
                println!("{} {}: {}", now, format!("{:?}", priority).bright_white().bold(), message)
            },
        }
    }

    // POSIX: 0 - Emergency
    pub fn emerg(&self, message: &str) {
        self.logging(&LogPriority::EMERG, &message)
    }
    
    /** @deprecated use `emerg(message:)` instead. */
    pub fn panic(&self, message: &str) {
        self.logging(&LogPriority::EMERG, &message)
    }
    
    // POSIX: 1 - Alert
    pub fn alert(&self, message: &str) {
        self.logging(&LogPriority::ALERT, &message)
    }
    
    // POSIX: 2 - Critical
    pub fn crit(&self, message: &str) {
        self.logging(&LogPriority::CRITICAL, &message)
    }
    
    // POSIX: 3 - Error
    pub fn err(&self, message: &str) {
        self.logging(&LogPriority::ERROR, &message)
    }
    
    /** @deprecated use `err(message:)` instead. */
    pub fn error(&self, message: &str) {
        self.logging(&LogPriority::ERROR, &message)
    }
    
    // POSIX: 4 - Warning
    pub fn warning(&self, message: &str) {
        self.logging(&LogPriority::WARNING, &message)
    }
    
    /** @deprecated use `warning(message:)` instead. */
    pub fn warn(&self, message: &str) {
        self.logging(&LogPriority::WARNING, &message)
    }
    
    // POSIX: 5 - Notice
    pub fn notice(&self, message: &str) {
        self.logging(&LogPriority::NOTICE, &message)
    }
    
    // POSIX: 6 - Informational
    pub fn info(&self, message: &str) {
        self.logging(&LogPriority::INFO, &message)
    }
    
    // POSIX: 7 - Debug
    pub fn debug(&self, message: &str) {
        self.logging(&LogPriority::DEBUG, &message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[fixture]
    fn message() -> String {
        String::from("Hello, UNiD!")
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