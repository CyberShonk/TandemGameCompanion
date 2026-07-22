use crate::config::{ControlSelector, WindowTitleMatcher};
use crate::error::AppError;

#[derive(Debug)]
pub struct MatchedControl {
    pub window_title: String,
    pub control_id: i32,
    pub class_name: String,
}

#[cfg(windows)]
pub fn protect_guardian_status_channel() -> Result<(), AppError> {
    use std::os::windows::io::AsRawHandle;
    use windows::Win32::Foundation::{
        HANDLE, HANDLE_FLAG_INHERIT, HANDLE_FLAGS, SetHandleInformation,
    };

    let stdout = std::io::stdout();
    let handle = HANDLE(stdout.as_raw_handle());

    // SAFETY: the handle is borrowed from the live process stdout stream. Clearing its inherit
    // flag does not close or otherwise invalidate it; it only prevents future child processes
    // from receiving the guardian status channel.
    unsafe { SetHandleInformation(handle, HANDLE_FLAG_INHERIT.0, HANDLE_FLAGS(0)) }.map_err(
        |source| {
            AppError::runtime(format!(
                "could not protect the guardian status channel from child inheritance: {source}"
            ))
        },
    )
}

#[cfg(not(windows))]
pub fn protect_guardian_status_channel() -> Result<(), AppError> {
    Ok(())
}

#[cfg(windows)]
pub struct ProcessWaiter {
    pid: u32,
    handle: windows::Win32::Foundation::HANDLE,
}

#[cfg(windows)]
impl ProcessWaiter {
    pub fn open(pid: u32) -> Result<Self, AppError> {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE};

        // SAFETY: OpenProcess is called with a valid access mask, no handle inheritance,
        // and a process ID reported by the worker immediately after process creation.
        let handle = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, false, pid) }.map_err(|source| {
            AppError::runtime(format!(
                "guardian could not open game process {pid} for synchronization: {source}"
            ))
        })?;

        Ok(Self { pid, handle })
    }

    pub fn wait(&self) -> Result<(), AppError> {
        use windows::Win32::Foundation::WAIT_OBJECT_0;
        use windows::Win32::System::Threading::{INFINITE, WaitForSingleObject};

        // SAFETY: self.handle is owned by this ProcessWaiter, remains valid for this call,
        // and was opened with PROCESS_SYNCHRONIZE access.
        let result = unsafe { WaitForSingleObject(self.handle, INFINITE) };
        if result == WAIT_OBJECT_0 {
            Ok(())
        } else {
            Err(AppError::runtime(format!(
                "guardian wait for game process {} returned {result:?}",
                self.pid
            )))
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }
}

#[cfg(windows)]
impl Drop for ProcessWaiter {
    fn drop(&mut self) {
        use windows::Win32::Foundation::CloseHandle;

        // SAFETY: this handle was returned by OpenProcess and is closed exactly once here.
        let _ = unsafe { CloseHandle(self.handle) };
    }
}

#[cfg(target_os = "linux")]
pub struct ProcessWaiter {
    pid: u32,
}

#[cfg(target_os = "linux")]
impl ProcessWaiter {
    pub fn open(pid: u32) -> Result<Self, AppError> {
        Ok(Self { pid })
    }

    pub fn wait(&self) -> Result<(), AppError> {
        use std::path::PathBuf;
        use std::thread;
        use std::time::Duration;

        let process_path = PathBuf::from("/proc").join(self.pid.to_string());
        while process_path.exists() {
            thread::sleep(Duration::from_millis(100));
        }
        Ok(())
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
pub struct ProcessWaiter {
    pid: u32,
}

#[cfg(not(any(windows, target_os = "linux")))]
impl ProcessWaiter {
    pub fn open(pid: u32) -> Result<Self, AppError> {
        Ok(Self { pid })
    }

    pub fn wait(&self) -> Result<(), AppError> {
        Err(AppError::runtime(format!(
            "guardian process waiting is not implemented on this platform for PID {}",
            self.pid
        )))
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }
}

#[cfg(windows)]
pub fn confirm_before_game(tool_name: &str) -> Result<bool, AppError> {
    use windows::Win32::UI::WindowsAndMessaging::{
        IDCANCEL, IDOK, MB_ICONINFORMATION, MB_OKCANCEL, MB_SETFOREGROUND, MB_TOPMOST, MessageBoxW,
    };
    use windows::core::PCWSTR;

    let message = format!(
        "Configure {tool_name}, then select OK to launch the game.\n\nSelect Cancel to stop the Tandem session."
    );
    let message: Vec<u16> = message.encode_utf16().chain(std::iter::once(0)).collect();
    let title: Vec<u16> = "Tandem Game Companion"
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();

    // SAFETY: both UTF-16 buffers are NUL-terminated and remain alive for the call. The dialog
    // has no owner window, so it does not disable or take ownership of the launched tool.
    let result = unsafe {
        MessageBoxW(
            None,
            PCWSTR(message.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OKCANCEL | MB_ICONINFORMATION | MB_SETFOREGROUND | MB_TOPMOST,
        )
    };

    if result == IDOK {
        Ok(true)
    } else if result == IDCANCEL {
        Ok(false)
    } else {
        Err(AppError::runtime(format!(
            "the before-game confirmation dialog returned {result:?}"
        )))
    }
}

#[cfg(not(windows))]
pub fn confirm_before_game(tool_name: &str) -> Result<bool, AppError> {
    use std::io::{self, Write};

    print!("Configure {tool_name}, then press Enter to launch the game. Type 'cancel' to stop: ");
    io::stdout()
        .flush()
        .map_err(|source| AppError::io("could not display the confirmation prompt", source))?;

    let mut response = String::new();
    let bytes = io::stdin()
        .read_line(&mut response)
        .map_err(|source| AppError::io("could not read the confirmation response", source))?;
    if bytes == 0 {
        return Ok(false);
    }

    Ok(!response.trim().eq_ignore_ascii_case("cancel"))
}

#[cfg(windows)]
pub fn find_top_level_window(
    pid: u32,
    matcher: &WindowTitleMatcher,
) -> Result<Option<String>, AppError> {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };
    use windows::core::BOOL;

    struct SearchContext {
        pid: u32,
        matcher: *const WindowTitleMatcher,
        matched_title: Option<String>,
    }

    unsafe extern "system" fn inspect_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: EnumWindows invokes this callback synchronously while the SearchContext passed
        // through LPARAM remains alive and exclusively borrowed by find_top_level_window.
        let context = unsafe { &mut *(lparam.0 as *mut SearchContext) };

        if context.matched_title.is_some() || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return BOOL(1);
        }

        let mut window_pid = 0;
        let _ = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut window_pid)) };
        if window_pid != context.pid {
            return BOOL(1);
        }

        let mut buffer = [0_u16; 1024];
        let length = unsafe { GetWindowTextW(hwnd, &mut buffer) };
        if length <= 0 {
            return BOOL(1);
        }

        let title = String::from_utf16_lossy(&buffer[..length as usize]);
        // SAFETY: matcher points to the borrowed matcher that remains alive for the synchronous
        // EnumWindows call.
        let matcher = unsafe { &*context.matcher };
        if matcher.matches(&title) {
            context.matched_title = Some(title);
        }

        BOOL(1)
    }

    let mut context = SearchContext {
        pid,
        matcher,
        matched_title: None,
    };
    let context_pointer = &mut context as *mut SearchContext;

    // SAFETY: inspect_window has the required system callback ABI. EnumWindows is synchronous,
    // and context_pointer remains valid and uniquely borrowed until enumeration completes.
    unsafe { EnumWindows(Some(inspect_window), LPARAM(context_pointer as isize)) }.map_err(
        |source| {
            AppError::runtime(format!(
                "could not enumerate top-level windows for companion process {pid}: {source}"
            ))
        },
    )?;

    Ok(context.matched_title)
}

#[cfg(not(windows))]
pub fn find_top_level_window(
    _pid: u32,
    _matcher: &WindowTitleMatcher,
) -> Result<Option<String>, AppError> {
    Err(AppError::runtime(
        "wait-for-window preparation is only available in Windows builds",
    ))
}

#[cfg(windows)]
pub fn find_descendant_control(
    pid: u32,
    window_matcher: &WindowTitleMatcher,
    control_selector: &ControlSelector,
) -> Result<Option<MatchedControl>, AppError> {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumChildWindows, EnumWindows, GetClassNameW, GetDlgCtrlID, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible,
    };
    use windows::core::BOOL;

    struct SearchContext {
        pid: u32,
        window_matcher: *const WindowTitleMatcher,
        control_selector: *const ControlSelector,
        matched_control: Option<MatchedControl>,
    }

    struct ChildSearchContext {
        pid: u32,
        control_selector: *const ControlSelector,
        matched_control: Option<(i32, String)>,
    }

    unsafe extern "system" fn inspect_control(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: EnumChildWindows invokes this callback synchronously while ChildSearchContext
        // remains alive and exclusively borrowed by inspect_window.
        let context = unsafe { &mut *(lparam.0 as *mut ChildSearchContext) };

        if context.matched_control.is_some()
            || !unsafe { IsWindowVisible(hwnd) }.as_bool()
            || !unsafe { IsWindowEnabled(hwnd) }.as_bool()
        {
            return BOOL(1);
        }

        let mut control_pid = 0;
        let _ = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut control_pid)) };
        if control_pid != context.pid {
            return BOOL(1);
        }

        let control_id = unsafe { GetDlgCtrlID(hwnd) };
        let mut class_buffer = [0_u16; 257];
        let class_length = unsafe { GetClassNameW(hwnd, &mut class_buffer) };
        if class_length <= 0 {
            return BOOL(1);
        }

        let class_name = String::from_utf16_lossy(&class_buffer[..class_length as usize]);
        // SAFETY: control_selector points to the borrowed selector that remains alive for the
        // synchronous EnumWindows and EnumChildWindows calls.
        let control_selector = unsafe { &*context.control_selector };
        if control_selector.matches(control_id, &class_name) {
            context.matched_control = Some((control_id, class_name));
        }

        BOOL(1)
    }

    unsafe extern "system" fn inspect_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: EnumWindows invokes this callback synchronously while SearchContext remains
        // alive and exclusively borrowed by find_descendant_control.
        let context = unsafe { &mut *(lparam.0 as *mut SearchContext) };

        if context.matched_control.is_some() || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return BOOL(1);
        }

        let mut window_pid = 0;
        let _ = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut window_pid)) };
        if window_pid != context.pid {
            return BOOL(1);
        }

        let mut title_buffer = [0_u16; 1024];
        let title_length = unsafe { GetWindowTextW(hwnd, &mut title_buffer) };
        if title_length <= 0 {
            return BOOL(1);
        }

        let window_title = String::from_utf16_lossy(&title_buffer[..title_length as usize]);
        // SAFETY: window_matcher points to the borrowed matcher that remains alive for the
        // synchronous EnumWindows call.
        let window_matcher = unsafe { &*context.window_matcher };
        if !window_matcher.matches(&window_title) {
            return BOOL(1);
        }

        let mut child_context = ChildSearchContext {
            pid: context.pid,
            control_selector: context.control_selector,
            matched_control: None,
        };
        let child_context_pointer = &mut child_context as *mut ChildSearchContext;

        // SAFETY: inspect_control has the required callback ABI. EnumChildWindows is synchronous,
        // hwnd is a live top-level window supplied by EnumWindows, and child_context_pointer
        // remains valid and uniquely borrowed until child enumeration completes.
        let _ = unsafe {
            EnumChildWindows(
                Some(hwnd),
                Some(inspect_control),
                LPARAM(child_context_pointer as isize),
            )
        };

        if let Some((control_id, class_name)) = child_context.matched_control {
            context.matched_control = Some(MatchedControl {
                window_title,
                control_id,
                class_name,
            });
        }

        BOOL(1)
    }

    let mut context = SearchContext {
        pid,
        window_matcher,
        control_selector,
        matched_control: None,
    };
    let context_pointer = &mut context as *mut SearchContext;

    // SAFETY: inspect_window has the required system callback ABI. EnumWindows is synchronous,
    // and context_pointer remains valid and uniquely borrowed until enumeration completes.
    unsafe { EnumWindows(Some(inspect_window), LPARAM(context_pointer as isize)) }.map_err(
        |source| {
            AppError::runtime(format!(
                "could not enumerate top-level windows for companion process {pid}: {source}"
            ))
        },
    )?;

    Ok(context.matched_control)
}

#[cfg(not(windows))]
pub fn find_descendant_control(
    _pid: u32,
    _window_matcher: &WindowTitleMatcher,
    _control_selector: &ControlSelector,
) -> Result<Option<MatchedControl>, AppError> {
    Err(AppError::runtime(
        "wait-for-control preparation is only available in Windows builds",
    ))
}
