use crate::{battery, extra, Fail, READOUTS};
use bytesize::ByteSize;
use macchina_read::traits::{KernelReadout, GeneralReadout, MemoryReadout, ReadoutError, BatteryReadout};

/// Construct a new _String_ from the value
/// returned by `read::uptime`
pub fn uptime(shorthand: bool) -> Result<String, ReadoutError> {
    let mut formatted_uptime = String::new();
    let uptime: f32 = READOUTS.general.uptime()?.parse().unwrap();
    // Uptime is formatted to dd:hh:mm if the system has been up for longer than 60 seconds
    if uptime > 60.0 {
        let up_days = (uptime / 60.0 / 60.0 / 24.0).floor();
        let up_hours = (uptime / 60.0 / 60.0 % 24.0).floor();
        let up_minutes = (uptime / 60.0 % 60.0).floor();
        match shorthand {
            true => {
                if up_days != 0.0 {
                    formatted_uptime.push_str(&up_days.to_string());
                    formatted_uptime.push_str("d ");
                }
                if up_hours != 0.0 {
                    formatted_uptime.push_str(&up_hours.to_string());
                    formatted_uptime.push_str("h ");
                }
                if up_minutes != 0.0 {
                    formatted_uptime.push_str(&up_minutes.to_string());
                    formatted_uptime.push_str("m");
                }
            }
            false => {
                if up_days != 0.0 {
                    if up_days == 1.0 {
                        formatted_uptime.push_str(&up_days.to_string());
                        formatted_uptime.push_str(" day ");
                    } else {
                        formatted_uptime.push_str(&up_days.to_string());
                        formatted_uptime.push_str(" days ");
                    }
                }
                if up_hours != 0.0 {
                    if up_hours == 1.0 {
                        formatted_uptime.push_str(&up_hours.to_string());
                        formatted_uptime.push_str(" hour ");
                    } else {
                        formatted_uptime.push_str(&up_hours.to_string());
                        formatted_uptime.push_str(" hours ");
                    }
                }
                if up_minutes != 0.0 {
                    if up_minutes == 1.0 {
                        formatted_uptime.push_str(&up_minutes.to_string());
                        formatted_uptime.push_str(" minute");
                    } else {
                        formatted_uptime.push_str(&up_minutes.to_string());
                        formatted_uptime.push_str(" minutes");
                    }
                }
            }
        }
    }
    // Uptime is formatted to seconds only if the system has been up for fewer than 60 seconds
    else {
        let up_seconds = (uptime % 60.0).floor();
        if up_seconds != 0.0 {
            formatted_uptime = up_seconds.to_string();
            formatted_uptime.push_str("s");
        }
    }

    Ok(formatted_uptime.trim().to_string())
}

/// Construct a new _String_ from the values
/// returned by `read::hostname` and `read::username`
pub fn host() -> Result<String, ReadoutError> {
    let username = READOUTS.general.username()?;
    let hostname = READOUTS.general.hostname()?;

    Ok(format!("{}@{}", username, hostname))
}

#[cfg(target_os = "linaux")]
/// Construct a new _String_ from the values
/// returned by `read::battery_percentage` and `read::battery_status`
pub fn battery(fail: &mut Fail) -> String {
    let percentage = battery::percentage(fail);
    let status = battery::status(fail);
    if !percentage.is_empty() && !status.is_empty() {
        if percentage != "100" {
            return String::from(percentage + "% & " + &status);
        }
        return status;
    }
    String::from("Unknown")
}

/// Construct a new _String_ from the values
/// returned by `read::battery_percentage` and `read::battery_status`
pub fn battery() -> Result<String, ReadoutError> {
    let percentage = READOUTS.battery.percentage()?;
    let status_from_read_func = READOUTS.battery.status()?;
    if !percentage.is_empty() && !status_from_read_func.is_empty() {
        // Holds either "Charging" or "Discharging" values
        return if percentage != "100" {
            if status_from_read_func == "TRUE" {
                Ok(format!("{}% & Charging", percentage))
            } else {
                Ok(format!("{}% & Discharging", percentage))
            }
        } else {
            Ok(String::from("Full"))
        }
    }

    Err(ReadoutError::MetricNotAvailable)
}

/// Construct a new _String_ from the values
/// returned by `memory::used` and `memory::memtotal`
pub fn memory() -> Result<String, ReadoutError> {
    let total = ByteSize::kb(READOUTS.memory.total()?);
    let used = ByteSize::kb(READOUTS.memory.used()?);

    Ok(format!("{}/{}", used, total))
}

/// Construct a new _String_ from the values
/// returned by `read::cpu_model_name` and `num_cpus::get`
pub fn cpu() -> Result<String, ReadoutError> {
    let cpu_model = READOUTS.general.cpu_model_name()?;

    Ok(format!("{} ({})", cpu_model, num_cpus::get())
        .replace("(TM)", "™")
        .replace("(R)", "®"))
}

/// Construct a new _String_ from the values
/// returned by `product::sys_vendor` and `product::product_family` or `product::product_version`
#[cfg(target_os = "linux")]
pub fn machine() -> String {
    if product::product_family() == product::product_name()
        && product::product_family() == product::product_version()
    {
        return product::product_family();
    } else if product::product_version().is_empty() || product::product_version().len() <= 15 {
        return String::from(
            product::product_vendor()
                + " "
                + &product::product_family()
                + " "
                + &product::product_name(),
        );
    }
    product::product_version()
}

#[cfg(target_os = "netbsd")]
/// Construct a new _String_ from the values
/// returned by `product::sys_vendor` and `product::product_family` or `product::product_version`
pub fn machine() -> String {
    if product::system_version() == product::system_product()
        && product::system_version() == product::system_vendor()
    {
        return product::system_vendor();
    }
    product::system_vendor() + " " + &product::system_product() + " " + &product::system_version()
}

/// Similar to how basename works
pub fn desktop_environment(mut session_name: String) -> String {
    let last_occurence_index = session_name.rfind("/").unwrap() + 1;
    session_name.replace_range(0..last_occurence_index, "");
    return extra::ucfirst(&session_name);
}

/// Returns a concatenated string of the kernel name and its release
pub fn kernel() -> Result<String, ReadoutError> {
    let os_type = READOUTS.kernel.os_type().unwrap_or(String::new());
    let os_release = READOUTS.kernel.os_release().unwrap_or(String::new());

    if !(os_type.is_empty() || os_release.is_empty()) {
        return Ok(format!("{} {}", os_type, os_release));
    }

    Err(ReadoutError::MetricNotAvailable)
}
