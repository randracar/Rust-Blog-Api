use chrono::prelude::*;

pub fn format_time() -> String {
    let date: DateTime<Local> = Local::now();
    let fdt = date.format_localized("%A %e %B %Y %T", Locale::en_GB).to_string();
    fdt
}