use crate::quota::QuotaSnapshot;

pub const ICON_SIZE: usize = 32;
const BACKGROUND: [u8; 4] = [27, 30, 35, 255];
const TRACK: [u8; 4] = [58, 64, 73, 255];
const UNKNOWN: [u8; 4] = [146, 154, 167, 255];
const FIVE_COLOR: [u8; 4] = [123, 226, 185, 255];
const SEVEN_COLOR: [u8; 4] = [176, 163, 245, 255];

pub fn remaining(used: Option<f64>) -> Option<f64> {
    used.filter(|value| value.is_finite() && (0.0..=100.0).contains(value))
        .map(|value| 100.0 - value)
}

pub fn tooltip(snapshot: &QuotaSnapshot) -> String {
    let value = |used: Option<f64>| match remaining(used) {
        Some(percent) => format!("{percent:.0}%"),
        None => "暂无数据".to_string(),
    };
    format!(
        "CodexQuota\n5h：{}\n7d：{}{}",
        value(snapshot.five_hour.as_ref().and_then(|q| q.used_percent)),
        value(snapshot.seven_day.as_ref().and_then(|q| q.used_percent)),
        if snapshot.stale {
            "\n缓存数据，可能已过期"
        } else {
            ""
        }
    )
}

/// Draws two horizontal quota progress bars: y=8 for 5h and y=20 for 7d.
/// Unknown values use a neutral short marker, never a full green bar.
pub fn render(snapshot: &QuotaSnapshot) -> Vec<u8> {
    let mut rgba = vec![0; ICON_SIZE * ICON_SIZE * 4];
    fill_rect(&mut rgba, 0, 0, ICON_SIZE, ICON_SIZE, BACKGROUND);
    draw_bar(
        &mut rgba,
        4,
        8,
        24,
        remaining(snapshot.five_hour.as_ref().and_then(|q| q.used_percent)),
        FIVE_COLOR,
    );
    draw_bar(
        &mut rgba,
        4,
        20,
        24,
        remaining(snapshot.seven_day.as_ref().and_then(|q| q.used_percent)),
        SEVEN_COLOR,
    );
    rgba
}

fn draw_bar(
    rgba: &mut [u8],
    x: usize,
    y: usize,
    width: usize,
    remaining: Option<f64>,
    normal_color: [u8; 4],
) {
    fill_rect(rgba, x, y, width, 4, TRACK);
    let (filled, color) = match remaining {
        Some(percent) => {
            let color = if percent < 20.0 {
                [255, 133, 133, 255]
            } else {
                normal_color
            };
            (((percent / 100.0) * width as f64).round() as usize, color)
        }
        None => (3, UNKNOWN),
    };
    fill_rect(rgba, x, y, filled, 4, color);
}

fn fill_rect(rgba: &mut [u8], x: usize, y: usize, width: usize, height: usize, color: [u8; 4]) {
    for row in y..(y + height).min(ICON_SIZE) {
        for col in x..(x + width).min(ICON_SIZE) {
            let offset = (row * ICON_SIZE + col) * 4;
            rgba[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn out_of_range_percentages_stay_unknown() {
        for value in [-1.0, 101.0, f64::NAN, f64::INFINITY] {
            assert!(remaining(Some(value)).is_none());
        }
    }
    use crate::quota::{QuotaSnapshot, WindowQuota};

    fn snapshot(five_used: Option<f64>, seven_used: Option<f64>) -> QuotaSnapshot {
        QuotaSnapshot {
            account: None,
            five_hour: Some(WindowQuota {
                label: "5h".into(),
                used_percent: five_used,
                reset_at: None,
            }),
            seven_day: Some(WindowQuota {
                label: "7d".into(),
                used_percent: seven_used,
                reset_at: None,
            }),
            updated_at: None,
            source: "test".into(),
            stale: false,
            error: None,
            accounts: Vec::new(),
        }
    }

    #[test]
    fn renders_two_independent_progress_bars() {
        let pixels = render(&snapshot(Some(25.0), Some(75.0)));
        let upper = &pixels[(8 * ICON_SIZE + 4) * 4..(8 * ICON_SIZE + 4) * 4 + 4];
        let lower = &pixels[(20 * ICON_SIZE + 4) * 4..(20 * ICON_SIZE + 4) * 4 + 4];
        assert_eq!(upper, FIVE_COLOR);
        assert_eq!(lower, SEVEN_COLOR);
        assert_ne!(
            pixels[(8 * ICON_SIZE + 22) * 4..(8 * ICON_SIZE + 22) * 4 + 4],
            FIVE_COLOR
        );
    }

    #[test]
    fn unknown_quota_does_not_render_as_full() {
        let pixels = render(&snapshot(None, None));
        assert_eq!(
            &pixels[(8 * ICON_SIZE + 4) * 4..(8 * ICON_SIZE + 4) * 4 + 4],
            UNKNOWN
        );
        assert_eq!(
            &pixels[(8 * ICON_SIZE + 12) * 4..(8 * ICON_SIZE + 12) * 4 + 4],
            TRACK
        );
    }
}
