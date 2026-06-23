#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TsGranularity {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
}

pub fn ts_prefix(ts_ms: i64, epoch_ms: i64, granularity: TsGranularity) -> alloc::string::String {
    let ms_per_unit = match granularity {
        TsGranularity::Second => 1000i64,
        TsGranularity::Minute => 60i64 * 1000,
        TsGranularity::Hour => 3600i64 * 1000,
        TsGranularity::Day => 24i64 * 3600 * 1000,
        TsGranularity::Week => 7i64 * 24 * 3600 * 1000,
        TsGranularity::Month => 30i64 * 24 * 3600 * 1000,
    };

    let units = ((ts_ms - epoch_ms) / ms_per_unit).max(0) as usize;
    let vowels: [char; 5] = ['A', 'E', 'I', 'O', 'U'];
    let first = vowels[(units / 5) % 5];
    let second = vowels[units % 5];

    alloc::format!("K{}-R{}", first, second)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_offset() {
        assert_eq!(
            ts_prefix(1_700_000_000_000, 1_700_000_000_000, TsGranularity::Month),
            "KA-RA"
        );
    }

    #[test]
    fn one_month_offset() {
        let epoch = 1_700_000_000_000i64;
        let ms = 30i64 * 24 * 3600 * 1000;
        assert_eq!(ts_prefix(epoch + ms, epoch, TsGranularity::Month), "KA-RE");
    }
}
