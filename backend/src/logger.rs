use colored::Colorize;

pub enum LoggerLevel {
    Info,
    Error,
}

pub fn log(level: LoggerLevel, message: String) {
    let level = match level {
        LoggerLevel::Info => " INF ".on_blue(),
        LoggerLevel::Error => " ERR ".on_red(),
    };

    let time = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    println!("{} {} {}", time.bright_black(), level, message);
}
