// FIXME: Make me pass! Diff budget: 25 lines.

#[derive(Debug)]
enum Duration {
    MilliSeconds(u64),
    Seconds(u32),
    Minutes(u16),
}

impl Duration {
    fn to_ms(&self) -> u64 {
        match *self {
            Self::MilliSeconds(ms) => ms,
            Self::Seconds(s) => u64::from(s) * 1000,
            Self::Minutes(m) => u64::from(m) * 60 * 1000,
        }
    }
}

impl PartialEq for Duration {
    fn eq(&self, other: &Self) -> bool {
        self.to_ms() == other.to_ms()
    }
}

// What traits does `Duration` need to implement?

#[test]
fn traits() {
    assert_eq!(Duration::Seconds(120), Duration::Minutes(2));
    assert_eq!(Duration::Seconds(420), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(420000), Duration::Minutes(7));
    assert_eq!(Duration::MilliSeconds(43000), Duration::Seconds(43));
}
