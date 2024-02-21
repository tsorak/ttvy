use colored::*;

pub(crate) fn warn(s: &str) {
    println!("{}", s.yellow());
}

pub(crate) mod chat {
    use colored::Colorize;

    pub fn color_status(b: bool) {
        if b {
            println!("[{}] Colored names", "ON".green());
        } else {
            println!("[{}] Colored names", "OFF".red());
        }
    }

    pub fn pad_status(b: bool) {
        if b {
            println!("[{}] Newline padding", "ON".green());
        } else {
            println!("[{}] Newline padding", "OFF".red());
        }
    }
}
