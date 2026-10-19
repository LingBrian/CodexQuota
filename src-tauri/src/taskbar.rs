use serde::{Deserialize, Serialize};
use tauri::Manager;
#[cfg(not(windows))]
use tauri::{PhysicalPosition, PhysicalSize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct OverlaySettings {
    #[serde(default = "legacy_layout_version")]
    pub layout_version: u8,
    pub show_five: bool,
    pub show_seven: bool,
    pub reverse: bool,
    pub font_weight: u16,
    pub background: String,
    pub background_opacity: f32,
    pub display_style: String,
    pub corner_radius: u8,
    pub gap: u16,
    pub edge_offset: i32,
    pub placement: String,
    pub offset_x: i32,
    pub offset_y: i32,
    pub monitor: Option<usize>,
    pub width: u16,
    pub height: u16,
    pub click_through: bool,
    pub show_countdown: bool,
    pub show_used: bool,
    pub progress: bool,
    pub low_color: String,
}
fn legacy_layout_version() -> u8 {
    0
}
impl Default for OverlaySettings {
    fn default() -> Self {
        Self {
            layout_version: 1,
            show_five: true,
            show_seven: true,
            reverse: false,
            font_weight: 600,
            background: "#1b1e23".into(),
            background_opacity: 0.9,
            display_style: "bars".into(),
            corner_radius: 10,
            gap: 16,
            edge_offset: 0,
            placement: "taskbar".into(),
            offset_x: 64,
            offset_y: 64,
            monitor: None,
            width: 420,
            height: 48,
            click_through: false,
            show_countdown: true,
            show_used: false,
            progress: true,
            low_color: "#ff7373".into(),
        }
    }
}
impl OverlaySettings {
    pub fn validate(&self) -> Result<(), String> {
        for color in [&self.background, &self.low_color] {
            if color.len() != 7
                || !color.starts_with('#')
                || !color[1..].bytes().all(|b| b.is_ascii_hexdigit())
            {
                return Err("组件颜色须为 #RRGGBB".into());
            }
        }
        if !(120..=1000).contains(&self.width)
            || !(24..=160).contains(&self.height)
            || !(100..=900).contains(&self.font_weight)
            || self.gap > 100
            || self.edge_offset.unsigned_abs() > 10000
            || self.offset_x.unsigned_abs() > 20000
            || self.offset_y.unsigned_abs() > 20000
            || !["taskbar", "desktop"].contains(&self.placement.as_str())
            || !(0.0..=1.0).contains(&self.background_opacity)
            || self.corner_radius > 24
            || ![
                "capsule",
                "bars",
                "text",
                "light-glass",
                "dark-glass",
                "light-solid",
                "dark-solid",
            ]
            .contains(&self.display_style.as_str())
            || self.monitor.is_some_and(|index| index > 32)
        {
            return Err("任务栏组件尺寸、间距或透明度超出范围".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[cfg(test)]
pub fn position(work: Rect, width: u32, height: u32, offset: i32) -> (i32, i32) {
    let max_x =
        (i64::from(work.x) + i64::from(work.width) - i64::from(width)).max(i64::from(work.x));
    let max_y =
        (i64::from(work.y) + i64::from(work.height) - i64::from(height)).max(i64::from(work.y));
    (
        (max_x - i64::from(offset)).clamp(i64::from(work.x), max_x) as i32,
        (max_y - i64::from(offset)).clamp(i64::from(work.y), max_y) as i32,
    )
}
#[cfg(test)]
pub fn near_edge(work: Rect, full: Rect, width: u32, height: u32, offset: i32) -> (i32, i32) {
    let (mut x, mut y) = position(work, width, height, offset);
    if work.x > full.x {
        x = (i64::from(work.x) + i64::from(offset.max(0)))
            .min(i64::from(work.x) + (i64::from(work.width) - i64::from(width)).max(0))
            as i32;
    }
    if work.y > full.y {
        y = (i64::from(work.y) + i64::from(offset.max(0)))
            .min(i64::from(work.y) + (i64::from(work.height) - i64::from(height)).max(0))
            as i32;
    }
    (x, y)
}

/// Query the documented AppBar API without modifying Explorer's window tree.
#[cfg(windows)]
pub fn primary_taskbar() -> Option<Rect> {
    use windows_sys::Win32::UI::Shell::{SHAppBarMessage, ABM_GETTASKBARPOS, APPBARDATA};
    let mut data = APPBARDATA {
        cbSize: std::mem::size_of::<APPBARDATA>() as u32,
        ..Default::default()
    };
    // SAFETY: data is a correctly sized APPBARDATA, written only during this call.
    if unsafe { SHAppBarMessage(ABM_GETTASKBARPOS, &mut data) } == 0 {
        return None;
    }
    Some(Rect {
        x: data.rc.left,
        y: data.rc.top,
        width: (data.rc.right - data.rc.left).max(0) as u32,
        height: (data.rc.bottom - data.rc.top).max(0) as u32,
    })
}

fn contains(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && i64::from(inner.x) + i64::from(inner.width)
            <= i64::from(outer.x) + i64::from(outer.width)
        && i64::from(inner.y) + i64::from(inner.height)
            <= i64::from(outer.y) + i64::from(outer.height)
}

pub fn taskbar_band(work: Rect, full: Rect, primary: Option<Rect>) -> Option<Rect> {
    if let Some(bar) = primary.filter(|bar| bar.width > 0 && bar.height > 0 && contains(full, *bar))
    {
        return Some(bar);
    }
    if work.y > full.y {
        return Some(Rect {
            height: (work.y - full.y) as u32,
            ..full
        });
    }
    if work.x > full.x {
        return Some(Rect {
            width: (work.x - full.x) as u32,
            ..full
        });
    }
    let bottom = full.y + full.height as i32 - work.y - work.height as i32;
    if bottom > 0 {
        return Some(Rect {
            y: work.y + work.height as i32,
            height: bottom as u32,
            ..full
        });
    }
    let right = full.x + full.width as i32 - work.x - work.width as i32;
    if right > 0 {
        return Some(Rect {
            x: work.x + work.width as i32,
            width: right as u32,
            ..full
        });
    }
    None
}

pub fn in_taskbar(bar: Rect, width: u32, height: u32, offset: i32) -> Rect {
    let width = width.min(bar.width).max(1);
    let height = height.min(bar.height).max(1);
    let horizontal = bar.width >= bar.height;
    Rect {
        x: bar.x
            + if horizontal {
                offset.clamp(0, (bar.width - width) as i32)
            } else {
                ((bar.width - width) / 2) as i32
            },
        y: bar.y
            + if horizontal {
                ((bar.height - height) / 2) as i32
            } else {
                offset.clamp(0, (bar.height - height) as i32)
            },
        width,
        height,
    }
}

pub fn sync(app: &tauri::AppHandle, settings: &crate::settings::AppSettings) -> Result<(), String> {
    let Some(window) = app.get_webview_window("overlay") else {
        return Ok(());
    };
    let monitors = app.available_monitors().map_err(|_| "无法获取显示器")?;
    let selected = settings
        .overlay
        .monitor
        .and_then(|index| monitors.get(index))
        .cloned()
        .or_else(|| app.primary_monitor().ok().flatten())
        .ok_or("未找到显示器")?;
    let scale = selected.scale_factor();
    let work = selected.work_area();
    let width = ((f64::from(settings.overlay.width) * scale).round() as u32)
        .min(work.size.width)
        .max(1);
    let height = ((f64::from(settings.overlay.height) * scale).round() as u32)
        .min(work.size.height)
        .max(1);
    let rect = Rect {
        x: work.position.x,
        y: work.position.y,
        width: work.size.width,
        height: work.size.height,
    };
    let full = Rect {
        x: selected.position().x,
        y: selected.position().y,
        width: selected.size().width,
        height: selected.size().height,
    };
    #[cfg(windows)]
    let primary = primary_taskbar();
    #[cfg(not(windows))]
    let primary = None;
    let geometry = if settings.overlay.placement == "taskbar" {
        let bar = taskbar_band(rect, full, primary)
            .ok_or("此显示器未检测到任务栏，请选择其他显示器或桌面浮窗模式")?;
        in_taskbar(
            bar,
            width,
            height,
            (f64::from(settings.overlay.edge_offset) * scale).round() as i32,
        )
    } else {
        Rect {
            x: rect.x
                + ((f64::from(settings.overlay.offset_x) * scale).round() as i32)
                    .clamp(0, (rect.width - width) as i32),
            y: rect.y
                + ((f64::from(settings.overlay.offset_y) * scale).round() as i32)
                    .clamp(0, (rect.height - height) as i32),
            width,
            height,
        }
    };
    let Rect {
        x,
        y,
        width,
        height,
    } = geometry;
    #[cfg(not(windows))]
    window
        .set_size(PhysicalSize::new(width, height))
        .map_err(|_| "组件尺寸设置失败")?;
    #[cfg(not(windows))]
    window
        .set_position(PhysicalPosition::new(x, y))
        .map_err(|_| "组件位置设置失败")?;
    #[cfg(windows)]
    {
        use windows_sys::Win32::Foundation::RECT;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, GetWindowRect, SetWindowLongPtrW, SetWindowPos, GWL_EXSTYLE,
            GWL_STYLE, HWND_TOPMOST, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOSENDCHANGING,
            WS_CAPTION, WS_EX_APPWINDOW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
            WS_EX_TRANSPARENT, WS_THICKFRAME,
        };
        let hwnd = window.hwnd().map_err(|_| "无法获取组件窗口句柄")?.0
            as windows_sys::Win32::Foundation::HWND;
        // SAFETY: only our overlay is changed. SetWindowPos uses outer dimensions,
        // unlike the framework's client-size API, and never activates the window.
        unsafe {
            let current_extended = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            let mut extended =
                (current_extended | WS_EX_NOACTIVATE as isize | WS_EX_TOOLWINDOW as isize)
                    & !(WS_EX_APPWINDOW as isize);
            if settings.overlay.click_through {
                extended |= (WS_EX_LAYERED | WS_EX_TRANSPARENT) as isize;
            } else {
                extended &= !(WS_EX_TRANSPARENT as isize);
            }
            let extended_changed = current_extended != extended;
            if extended_changed {
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, extended);
            }
            let style = GetWindowLongPtrW(hwnd, GWL_STYLE);
            let target_style = style & !((WS_CAPTION | WS_THICKFRAME) as isize);
            let style_changed = style != target_style;
            if style_changed {
                SetWindowLongPtrW(hwnd, GWL_STYLE, target_style);
            }
            let mut current_rect = RECT::default();
            let geometry_matches = GetWindowRect(hwnd, &mut current_rect) != 0
                && current_rect.left == x
                && current_rect.top == y
                && current_rect.right - current_rect.left == width as i32
                && current_rect.bottom - current_rect.top == height as i32;
            // Reassert topmost only when the window actually needs a native
            // geometry/style update. Repeating this every monitor tick makes
            // Explorer redraw the taskbar; never changing it can leave the
            // overlay behind the taskbar after a shell z-order transition.
            if !geometry_matches || style_changed || extended_changed {
                let mut flags = SWP_NOACTIVATE | SWP_NOSENDCHANGING;
                if style_changed || extended_changed {
                    flags |= SWP_FRAMECHANGED;
                }
                if SetWindowPos(hwnd, HWND_TOPMOST, x, y, width as i32, height as i32, flags) == 0 {
                    return Err("组件原生边界设置失败".into());
                }
            }
        }
    }
    #[cfg(not(windows))]
    window
        .set_ignore_cursor_events(settings.overlay.click_through)
        .map_err(|_| "鼠标穿透设置失败")?;
    #[cfg(windows)]
    let _ = primary_taskbar();
    Ok(())
}

pub fn show_without_activation(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetWindowLongPtrW, SetWindowLongPtrW, SetWindowPos, ShowWindow, GWL_EXSTYLE,
            HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING, SWP_NOSIZE,
            SW_SHOWNOACTIVATE, WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
        };
        let hwnd = window.hwnd().map_err(|_| "无法获取组件窗口句柄")?.0
            as windows_sys::Win32::Foundation::HWND;
        // SAFETY: hwnd belongs to our own live Tauri window. Only documented
        // extended styles and ShowWindow are used; Explorer is not modified.
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd,
                GWL_EXSTYLE,
                (style | WS_EX_NOACTIVATE as isize | WS_EX_TOOLWINDOW as isize)
                    & !(WS_EX_APPWINDOW as isize),
            );
            if SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOSENDCHANGING,
            ) == 0
            {
                return Err("组件置顶失败".into());
            }
            ShowWindow(hwnd, SW_SHOWNOACTIVATE);
        }
        Ok(())
    }
    #[cfg(not(windows))]
    {
        window.show().map_err(|_| "组件显示失败".into())
    }
}

pub fn foreground_window_id() -> usize {
    #[cfg(windows)]
    {
        unsafe { windows_sys::Win32::UI::WindowsAndMessaging::GetForegroundWindow() as usize }
    }
    #[cfg(not(windows))]
    0
}

/// Return whether the current foreground window is actually above our overlay
/// in the desktop Z-order. A normal foreground change does not imply that the
/// overlay lost its position in the topmost band.
pub fn foreground_is_above(window: &tauri::WebviewWindow) -> Result<bool, String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetForegroundWindow, GetWindow, GW_HWNDPREV,
        };
        let hwnd =
            window.hwnd().map_err(|_| "组件句柄不可用")?.0 as windows_sys::Win32::Foundation::HWND;
        // SAFETY: GetWindow only traverses the desktop Z-order; no window state
        // is modified. Bound traversal in case a malformed chain is returned.
        unsafe {
            let foreground = GetForegroundWindow();
            if foreground.is_null() || foreground == hwnd {
                return Ok(false);
            }
            let mut current = GetWindow(hwnd, GW_HWNDPREV);
            for _ in 0..512 {
                if current.is_null() {
                    return Ok(false);
                }
                if current == foreground {
                    return Ok(true);
                }
                current = GetWindow(current, GW_HWNDPREV);
            }
        }
        Ok(false)
    }
    #[cfg(not(windows))]
    {
        let _ = window;
        Ok(false)
    }
}

pub fn reassert_topmost(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            SetWindowPos, ShowWindow, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSENDCHANGING,
            SWP_NOSIZE, SW_SHOWNOACTIVATE,
        };
        let hwnd =
            window.hwnd().map_err(|_| "组件句柄不可用")?.0 as windows_sys::Win32::Foundation::HWND;
        // Promote only on an actual foreground-app transition. No geometry or
        // frame change is requested, and the overlay is never activated.
        unsafe {
            if SetWindowPos(
                hwnd,
                HWND_TOPMOST,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_NOSENDCHANGING,
            ) == 0
            {
                return Err("组件置顶失败".into());
            }
            if !is_visible(window)? {
                ShowWindow(hwnd, SW_SHOWNOACTIVATE);
            }
        }
    }
    #[cfg(not(windows))]
    let _ = window;
    Ok(())
}

pub fn hide(window: &tauri::WebviewWindow) -> Result<(), String> {
    #[cfg(windows)]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
        let hwnd = window.hwnd().map_err(|_| "无法获取组件句柄")?.0
            as windows_sys::Win32::Foundation::HWND;
        // Match the native show path; do not rely on the framework's visibility cache.
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
        Ok(())
    }
    #[cfg(not(windows))]
    window.hide().map_err(|e| e.to_string())
}

pub fn is_visible(window: &tauri::WebviewWindow) -> Result<bool, String> {
    #[cfg(windows)]
    {
        let hwnd = window.hwnd().map_err(|_| "无法获取组件句柄")?.0
            as windows_sys::Win32::Foundation::HWND;
        // Read the actual native window state, including changes made by ShowWindow.
        Ok(unsafe { windows_sys::Win32::UI::WindowsAndMessaging::IsWindowVisible(hwnd) } != 0)
    }
    #[cfg(not(windows))]
    window.is_visible().map_err(|e| e.to_string())
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractionState {
    pub no_activate: bool,
    pub tool_window: bool,
    pub click_through: bool,
}
#[cfg(windows)]
pub fn interaction_state(window: &tauri::WebviewWindow) -> Result<InteractionState, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    };
    let hwnd =
        window.hwnd().map_err(|_| "组件句柄不可用")?.0 as windows_sys::Win32::Foundation::HWND;
    // SAFETY: reads documented extended styles of our own live overlay only.
    let style = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32;
    Ok(InteractionState {
        no_activate: style & WS_EX_NOACTIVATE != 0,
        tool_window: style & WS_EX_TOOLWINDOW != 0,
        click_through: style & WS_EX_TRANSPARENT != 0,
    })
}

#[cfg(windows)]
pub fn is_topmost(window: &tauri::WebviewWindow) -> Result<bool, String> {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowLongPtrW, GWL_EXSTYLE, WS_EX_TOPMOST,
    };
    let hwnd =
        window.hwnd().map_err(|_| "组件句柄不可用")?.0 as windows_sys::Win32::Foundation::HWND;
    // SAFETY: reads the documented topmost style of our own live overlay.
    Ok((unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32 & WS_EX_TOPMOST) != 0)
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MonitorInfo {
    index: usize,
    name: String,
    scale: f64,
}
#[tauri::command]
pub fn list_monitors(app: tauri::AppHandle) -> Result<Vec<MonitorInfo>, String> {
    Ok(app
        .available_monitors()
        .map_err(|_| "无法读取显示器")?
        .into_iter()
        .enumerate()
        .map(|(index, monitor)| MonitorInfo {
            index,
            name: monitor
                .name()
                .cloned()
                .unwrap_or_else(|| format!("显示器 {}", index + 1)),
            scale: monitor.scale_factor(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn taskbar_geometry_stays_inside_the_bar_and_moves_with_offset() {
        let full = Rect {
            x: -1920,
            y: 0,
            width: 1920,
            height: 1080,
        };
        let work = Rect {
            height: 1032,
            ..full
        };
        let bar = taskbar_band(work, full, None).unwrap();
        assert_eq!(bar.y, 1032);
        for offset in [-500, 0, 200, 10000] {
            assert!(contains(bar, in_taskbar(bar, 300, 160, offset)));
        }
        assert_eq!(
            in_taskbar(bar, 300, 46, 200).x - in_taskbar(bar, 300, 46, 100).x,
            100
        );
        let left = Rect {
            x: -1920,
            y: 0,
            width: 48,
            height: 1080,
        };
        assert!(contains(left, in_taskbar(left, 300, 46, 150)));
        assert!(taskbar_band(full, full, None).is_none());
    }
    #[test]
    fn disconnected_display_and_offset_values_are_validated() {
        let mut settings = OverlaySettings {
            monitor: Some(32),
            edge_offset: -500,
            ..Default::default()
        };
        assert!(settings.validate().is_ok());
        settings.monitor = Some(33);
        assert!(settings.validate().is_err());
        settings.monitor = None;
        settings.edge_offset = i32::MIN;
        assert!(settings.validate().is_err());
    }
    #[test]
    fn dpi_and_auto_hidden_taskbar_matrix() {
        for scale in [1.0_f64, 1.25, 1.5, 2.0] {
            for hidden in [false, true] {
                let full = Rect {
                    x: -1920,
                    y: -1080,
                    width: 1920,
                    height: 1080,
                };
                let work = Rect {
                    height: if hidden { 1080 } else { 1032 },
                    ..full
                };
                let width = (300.0 * scale).round() as u32;
                let height = (46.0 * scale).round() as u32;
                let (x, y) = near_edge(work, full, width, height, (12.0 * scale) as i32);
                assert!(x >= work.x && y >= work.y);
                assert!(
                    i64::from(x) + i64::from(width) <= i64::from(work.x) + i64::from(work.width)
                );
                assert!(
                    i64::from(y) + i64::from(height) <= i64::from(work.y) + i64::from(work.height)
                );
            }
        }
    }
    #[test]
    fn negative_coordinates_and_dpi_geometry_stay_in_work_area() {
        let work = Rect {
            x: -2560,
            y: 0,
            width: 2560,
            height: 1400,
        };
        assert_eq!(position(work, 600, 92, 24), (-624, 1284));
    }
    #[test]
    fn extreme_offsets_are_clamped() {
        let work = Rect {
            x: 0,
            y: 40,
            width: 1920,
            height: 1040,
        };
        assert_eq!(position(work, 300, 46, -500), (1620, 1034));
        assert_eq!(position(work, 300, 46, 5000), (0, 40));
    }
    #[test]
    fn invalid_style_is_rejected() {
        let style = OverlaySettings {
            width: 0,
            ..Default::default()
        };
        assert!(style.validate().is_err());
        let preset = OverlaySettings {
            display_style: "unknown-style".into(),
            ..Default::default()
        };
        assert!(preset.validate().is_err());
        let radius = OverlaySettings {
            corner_radius: 25,
            ..Default::default()
        };
        assert!(radius.validate().is_err());
    }
    #[test]
    fn follows_top_and_left_taskbars() {
        let full = Rect {
            x: 0,
            y: 0,
            width: 1920,
            height: 1080,
        };
        assert_eq!(
            near_edge(
                Rect {
                    x: 0,
                    y: 40,
                    width: 1920,
                    height: 1040
                },
                full,
                300,
                46,
                12
            )
            .1,
            52
        );
        assert_eq!(
            near_edge(
                Rect {
                    x: 80,
                    y: 0,
                    width: 1840,
                    height: 1080
                },
                full,
                300,
                46,
                12
            )
            .0,
            92
        );
    }
}
