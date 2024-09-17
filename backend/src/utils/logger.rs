use std::io::{Result, Write};
use chrono::Local;
use flexi_logger::{DeferredNow, Record};
use log::Level;
use colored::Colorize;

fn format_log_message(record: &Record) -> String {
    format!(
        "[{}] [{}]: {}",
        record.level(),
        Local::now().format("%H:%M %d/%m/%Y"),
        &record.args()
    )
}

pub fn format_log(w: &mut dyn Write, _d: &mut DeferredNow, record: &Record) -> Result<()> {
    write!(w, "{}", format_log_message(record))
}

pub fn format_colored_log(w: &mut dyn Write, _d: &mut DeferredNow, record: &Record) -> Result<()> {
    let log_message = format_log_message(record);

    write!(w, "{}", match record.level() {
        Level::Error => log_message.red(),
        Level::Warn => log_message.yellow(),
        Level::Info => log_message.cyan(),
        Level::Debug => log_message.green(),
        Level::Trace => log_message.purple()
    })
}
