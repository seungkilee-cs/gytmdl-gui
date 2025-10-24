use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use chrono::Utc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: String,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    DEBUG,
    INFO,
    WARN,
    ERROR,
}

pub struct DebugLogger {
    logs: Arc<Mutex<VecDeque<LogEntry>>>,
    max_logs: usize,
}

impl DebugLogger {
    pub fn new(max_logs: usize) -> Self {
        Self {
            logs: Arc::new(Mutex::new(VecDeque::new())),
            max_logs,
        }
    }

    pub fn log(&self, level: LogLevel, component: &str, message: &str, data: Option<serde_json::Value>) {
        // Print to console for development first
        let level_str = match level {
            LogLevel::DEBUG => "DEBUG",
            LogLevel::INFO => "INFO",
            LogLevel::WARN => "WARN",
            LogLevel::ERROR => "ERROR",
        };
        
        if let Some(ref data) = data {
            println!("[{}] {}: {} - {}", level_str, component, message, data);
        } else {
            println!("[{}] {}: {}", level_str, component, message);
        }

        let entry = LogEntry {
            timestamp: Utc::now().to_rfc3339(),
            level,
            component: component.to_string(),
            message: message.to_string(),
            data,
        };

        if let Ok(mut logs) = self.logs.lock() {
            logs.push_back(entry);
            
            // Keep only the most recent logs
            while logs.len() > self.max_logs {
                logs.pop_front();
            }
        }
    }

    pub fn debug(&self, component: &str, message: &str, data: Option<serde_json::Value>) {
        self.log(LogLevel::DEBUG, component, message, data);
    }

    pub fn info(&self, component: &str, message: &str, data: Option<serde_json::Value>) {
        self.log(LogLevel::INFO, component, message, data);
    }

    pub fn warn(&self, component: &str, message: &str, data: Option<serde_json::Value>) {
        self.log(LogLevel::WARN, component, message, data);
    }

    pub fn error(&self, component: &str, message: &str, data: Option<serde_json::Value>) {
        self.log(LogLevel::ERROR, component, message, data);
    }

    pub fn get_logs(&self) -> Vec<LogEntry> {
        if let Ok(logs) = self.logs.lock() {
            logs.iter().cloned().collect()
        } else {
            Vec::new()
        }
    }

    pub fn clear_logs(&self) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
        }
    }
}

// Global logger instance
lazy_static::lazy_static! {
    pub static ref DEBUG_LOGGER: DebugLogger = DebugLogger::new(1000);
}

// Convenience macros for logging
#[macro_export]
macro_rules! debug_log {
    ($component:expr, $message:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.debug($component, $message, None);
    };
    ($component:expr, $message:expr, $data:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.debug($component, $message, Some($data));
    };
}

#[macro_export]
macro_rules! info_log {
    ($component:expr, $message:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.info($component, $message, None);
    };
    ($component:expr, $message:expr, $data:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.info($component, $message, Some($data));
    };
}

#[macro_export]
macro_rules! warn_log {
    ($component:expr, $message:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.warn($component, $message, None);
    };
    ($component:expr, $message:expr, $data:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.warn($component, $message, Some($data));
    };
}

#[macro_export]
macro_rules! error_log {
    ($component:expr, $message:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.error($component, $message, None);
    };
    ($component:expr, $message:expr, $data:expr) => {
        crate::modules::debug_logger::DEBUG_LOGGER.error($component, $message, Some($data));
    };
}