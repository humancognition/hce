#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsGranularity {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

const MS_PER_DAY: i64 = 24 * 3600 * 1000;
const UNIX_EPOCH_YEAR: i64 = 1970;

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    match m {
        1 => 31,
        2 => {
            if is_leap_year(y) {
                29
            } else {
                28
            }
        }
        3 => 31,
        4 => 30,
        5 => 31,
        6 => 30,
        7 => 31,
        8 => 31,
        9 => 30,
        10 => 31,
        11 => 30,
        12 => 31,
        _ => 30,
    }
}

fn days_to_year_month(mut days: i64) -> (i64, i64) {
    let mut y = UNIX_EPOCH_YEAR;
    loop {
        let diy = if is_leap_year(y) { 366 } else { 365 };
        if days < diy {
            break;
        }
        days -= diy;
        y += 1;
    }
    let mut m = 1i64;
    loop {
        let dim = days_in_month(y, m);
        if days < dim {
            break;
        }
        days -= dim;
        m += 1;
    }
    (y, m)
}

fn calendar_months_since(ts_ms: i64, epoch_ms: i64) -> i64 {
    let ts_days = ts_ms / MS_PER_DAY;
    let ep_days = epoch_ms / MS_PER_DAY;
    let (ts_y, ts_m) = days_to_year_month(ts_days);
    let (ep_y, ep_m) = days_to_year_month(ep_days);
    (ts_y - ep_y) * 12 + (ts_m - ep_m)
}

pub fn ts_prefix(ts_ms: i64, epoch_ms: i64, granularity: TsGranularity) -> alloc::string::String {
    let units = match granularity {
        TsGranularity::Second => (ts_ms - epoch_ms) / 1000,
        TsGranularity::Minute => (ts_ms - epoch_ms) / (60 * 1000),
        TsGranularity::Hour => (ts_ms - epoch_ms) / (3600 * 1000),
        TsGranularity::Day => (ts_ms - epoch_ms) / MS_PER_DAY,
        TsGranularity::Week => (ts_ms - epoch_ms) / (7 * MS_PER_DAY),
        TsGranularity::Month => calendar_months_since(ts_ms, epoch_ms),
    };
    let units = units.max(0) as usize;
    let vowels: [char; 5] = ['A', 'E', 'I', 'O', 'U'];
    let first = vowels[(units / 5) % 5];
    let second = vowels[units % 5];

    alloc::format!("B{}-D{}", first, second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_offset() {
        assert_eq!(
            ts_prefix(1_700_000_000_000, 1_700_000_000_000, TsGranularity::Month),
            "BA-DA"
        );
    }

    #[test]
    fn calendar_month_different_months() {
        let epoch = 1_700_000_000_000i64;
        let mar30 = 1_711_756_800_000i64; // 2024-03-30
        let apr1 = 1_711_929_600_000i64; // 2024-04-01
        assert_ne!(
            ts_prefix(mar30, epoch, TsGranularity::Month),
            ts_prefix(apr1, epoch, TsGranularity::Month)
        );
    }

    #[test]
    fn calendar_month_monotonic() {
        let epoch = 1_700_000_000_000i64;
        let jan31 = 1_706_659_200_000i64; // 2024-01-31
        let feb1 = 1_706_745_600_000i64; // 2024-02-01
        let p1 = ts_prefix(jan31, epoch, TsGranularity::Month);
        let p2 = ts_prefix(feb1, epoch, TsGranularity::Month);
        assert!(p1 < p2, "jan31 {} < feb1 {}", p1, p2);
    }

    #[test]
    fn calendar_month_same_month_after_epoch_start() {
        let epoch = 1_700_000_000_000i64;
        let p1 = ts_prefix(epoch, epoch, TsGranularity::Month);
        let p2 = ts_prefix(epoch + 10 * 24 * 3600 * 1000, epoch, TsGranularity::Month);
        assert_eq!(p1, p2, "same month should give same prefix");
    }
}
