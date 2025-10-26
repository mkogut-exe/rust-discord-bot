use std::thread::sleep;
use std::time::{Duration, Instant};
use chrono::{Local, Utc, TimeZone, Offset};
use chrono_tz::Tz;

pub fn metric_clock(utc_timezone: i32) {
    let mut last_metric_second = 0;
    let timezone= utc_timezone * 3600;
    loop {
        let cycle_start = Instant::now();

        //let current_metric_todaystamp = get_metric_todaystamp(&timezone);
        let current_metric_second = get_metric_seconds(&timezone);
        let current_metric_minute = get_metric_minutes(&timezone);
        let current_metric_hour = get_metric_hours(&timezone);

        // Only print on a new metric second
        if current_metric_second != last_metric_second {
            let mut second_str = current_metric_second.to_string();
            let mut minute_str = current_metric_minute.to_string();
            let mut hour_str = current_metric_hour.to_string();
            if current_metric_second<9 {
                second_str = format!("0{}", current_metric_second);
            }
            if current_metric_minute<9 {
                minute_str = format!("0{}", current_metric_minute);
            }
            if current_metric_hour<9 {
                hour_str = format!("0{}", current_metric_hour);
            }
            println!("Metric Time: {}:{}:{}", hour_str, minute_str, second_str);
            last_metric_second = current_metric_second;
        }

        // Calculate how long this cycle took
        let cycle_time = cycle_start.elapsed();

        // Sleep for the remaining time to make each cycle exactly 864ms
        if cycle_time < Duration::from_nanos(864000000) {
            sleep(Duration::from_nanos(864000000) - cycle_time);
        }
    }
}

pub fn get_metric_time(utc_timezone: i32) -> (u64, u64, u64) {
    let timezone= utc_timezone * 3600;
    let metric_hour = get_metric_hours(&timezone);
    let metric_minute = get_metric_minutes(&timezone);
    let metric_second = get_metric_seconds(&timezone);
    (metric_hour, metric_minute, metric_second)
}

pub fn get_metric_time_string(utc_timezone: i32) -> String {
    let (h,m,s) = get_metric_time(utc_timezone);
    let mut second_str = s.to_string();
    let mut minute_str = m.to_string();
    let mut hour_str = h.to_string();
    if s<9 {
        second_str = format!("0{}", s);
    }
    if m<9 {
        minute_str = format!("0{}", m);
    }
    if h<9 {
        hour_str = format!("0{}", h);
    }
    format!("{}:{}:{}", hour_str, minute_str, second_str)
}
pub fn get_metric_h_m_string(utc_timezone: i32) -> String {
    let (h,m,s) = get_metric_time(utc_timezone);

    let mut minute_str = m.to_string();
    let mut hour_str = h.to_string();

    if m<9 {
        minute_str = format!("0{}", m);
    }
    if h<9 {
        hour_str = format!("0{}", h);
    }
    format!("{}:{}", hour_str, minute_str)
}
fn get_metric_todaystamp(timezone: &i32) -> u64 {
    let now_utc0 = Local::now();
    //println!("Current Time: {}", now.format("%H:%M:%S%.3f"));
    let timestamp_ms_utc0 = now_utc0.timestamp_millis(); // Milliseconds since Unix epoch in UTC

    let offset_ms =  timezone * 1000;
    let local_timestamp_ms = timestamp_ms_utc0 + offset_ms as i64;

    let ms_in_day = 86400000;
    let ms_since_midnight = local_timestamp_ms % ms_in_day;

    // Convert to metric seconds using integer arithmetic
    let metric_total = (ms_since_midnight as u64 * 100000) / ms_in_day as u64;

    metric_total
}


pub fn get_metric_seconds(timezone: &i32) -> u64 {
    let metric_timestamp = get_metric_todaystamp(timezone);
    metric_timestamp % 100
}
pub fn get_metric_minutes(timezone: &i32) -> u64 {
    let metric_timestamp = get_metric_todaystamp(timezone);
    (metric_timestamp / 100) % 100
}
pub fn get_metric_hours(timezone: &i32) -> u64 {
    let metric_timestamp = get_metric_todaystamp(timezone);
    metric_timestamp / 10000
}
pub fn get_utc_offset(tz: &Tz) -> String {
    let time_in_tz = Utc::now().with_timezone(tz);
    let offset_seconds = time_in_tz.offset().fix().local_minus_utc();
    let offset_hours = offset_seconds as f64 / 3600.0;

    let sign = if offset_hours >= 0.0 { '+' } else { '-' };
    format!("UTC{}{}", sign, offset_hours.abs())
}