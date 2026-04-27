use iced::Task;
use iced::window;

pub fn polish_window<M>(id: window::Id) -> Task<M>
where
    M: Send + 'static,
{
    window::run_with_handle(id, move |handle| {
        #[cfg(target_os = "windows")]
        windows_impl::apply_round_corners(handle);
        #[cfg(not(target_os = "windows"))]
        let _ = handle;
    })
    .discard()
}

#[cfg(target_os = "windows")]
mod windows_impl {
    use iced::window::raw_window_handle::{RawWindowHandle, WindowHandle};
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DWMWA_BORDER_COLOR, DWMWA_COLOR_NONE, DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_ROUND,
        DwmSetWindowAttribute,
    };

    pub fn apply_round_corners(handle: WindowHandle<'_>) {
        let RawWindowHandle::Win32(h) = handle.as_raw() else {
            return;
        };
        let hwnd: HWND = h.hwnd.get() as HWND;

        let pref: u32 = DWMWCP_ROUND as u32;
        let none: u32 = DWMWA_COLOR_NONE;
        unsafe {
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE as u32,
                &pref as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
            DwmSetWindowAttribute(
                hwnd,
                DWMWA_BORDER_COLOR as u32,
                &none as *const u32 as *const _,
                std::mem::size_of::<u32>() as u32,
            );
        }
    }
}
