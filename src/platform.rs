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

#[derive(Debug)]
pub struct ComboBoxSelection {
    pub window_title: String,
    pub control_id: i32,
    pub class_name: String,
    pub requested_index: u32,
    pub prior_index: Option<u32>,
    pub resulting_index: u32,
    pub notification_sent: bool,
}

#[derive(Debug)]
#[cfg_attr(not(windows), allow(dead_code))]
pub enum ComboBoxSelectionStatus {
    Pending { reason: String },
    Complete(ComboBoxSelection),
}

#[cfg(windows)]
#[derive(Clone)]
struct MatchedWindowHandle {
    hwnd: windows::Win32::Foundation::HWND,
    title: String,
}

#[cfg(windows)]
#[derive(Clone)]
struct MatchedControlHandle {
    hwnd: windows::Win32::Foundation::HWND,
    control_id: i32,
    class_name: String,
}

#[cfg(windows)]
fn matching_top_level_windows(
    pid: u32,
    matcher: &WindowTitleMatcher,
) -> Result<Vec<MatchedWindowHandle>, AppError> {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };
    use windows::core::BOOL;

    struct SearchContext {
        pid: u32,
        matcher: *const WindowTitleMatcher,
        matches: Vec<MatchedWindowHandle>,
        error: Option<String>,
    }

    unsafe extern "system" fn inspect_window(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: EnumWindows invokes this callback synchronously while SearchContext remains
        // alive and exclusively borrowed by matching_top_level_windows.
        let context = unsafe { &mut *(lparam.0 as *mut SearchContext) };

        if context.error.is_some() || !unsafe { IsWindowVisible(hwnd) }.as_bool() {
            return BOOL(1);
        }

        let mut window_pid = 0;
        let thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut window_pid)) };
        if thread_id == 0 {
            context.error =
                Some("GetWindowThreadProcessId failed during top-level window discovery".into());
            return BOOL(1);
        }
        if window_pid != context.pid {
            return BOOL(1);
        }

        let mut buffer = [0_u16; 1024];
        let length = unsafe { GetWindowTextW(hwnd, &mut buffer) };
        if length <= 0 {
            return BOOL(1);
        }

        let title = String::from_utf16_lossy(&buffer[..length as usize]);
        // SAFETY: matcher points to the borrowed matcher that remains alive for EnumWindows.
        let matcher = unsafe { &*context.matcher };
        if matcher.matches(&title) {
            context.matches.push(MatchedWindowHandle { hwnd, title });
        }

        BOOL(1)
    }

    let mut context = SearchContext {
        pid,
        matcher,
        matches: Vec::new(),
        error: None,
    };
    let context_pointer = &mut context as *mut SearchContext;

    // SAFETY: inspect_window has the required callback ABI. EnumWindows is synchronous, and the
    // context remains valid and uniquely borrowed until enumeration completes.
    unsafe { EnumWindows(Some(inspect_window), LPARAM(context_pointer as isize)) }.map_err(
        |source| {
            AppError::runtime(format!(
                "could not enumerate top-level windows for companion process {pid}: {source}"
            ))
        },
    )?;

    if let Some(error) = context.error {
        return Err(AppError::runtime(format!(
            "could not inspect top-level windows for companion process {pid}: {error}"
        )));
    }

    Ok(context.matches)
}

#[cfg(windows)]
fn matching_descendant_controls(
    parent: windows::Win32::Foundation::HWND,
    pid: u32,
    selector: &ControlSelector,
) -> Result<Vec<MatchedControlHandle>, AppError> {
    use windows::Win32::Foundation::{HWND, LPARAM};
    use windows::Win32::UI::Input::KeyboardAndMouse::IsWindowEnabled;
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumChildWindows, GetClassNameW, GetDlgCtrlID, GetWindowThreadProcessId, IsWindowVisible,
    };
    use windows::core::BOOL;

    struct SearchContext {
        pid: u32,
        selector: *const ControlSelector,
        matches: Vec<MatchedControlHandle>,
        error: Option<String>,
    }

    unsafe extern "system" fn inspect_control(hwnd: HWND, lparam: LPARAM) -> BOOL {
        // SAFETY: EnumChildWindows invokes this callback synchronously while SearchContext remains
        // alive and exclusively borrowed by matching_descendant_controls.
        let context = unsafe { &mut *(lparam.0 as *mut SearchContext) };

        if context.error.is_some()
            || !unsafe { IsWindowVisible(hwnd) }.as_bool()
            || !unsafe { IsWindowEnabled(hwnd) }.as_bool()
        {
            return BOOL(1);
        }

        let mut control_pid = 0;
        let thread_id = unsafe { GetWindowThreadProcessId(hwnd, Some(&mut control_pid)) };
        if thread_id == 0 {
            context.error =
                Some("GetWindowThreadProcessId failed during descendant-control discovery".into());
            return BOOL(1);
        }
        if control_pid != context.pid {
            return BOOL(1);
        }

        let control_id = unsafe { GetDlgCtrlID(hwnd) };
        let mut class_buffer = [0_u16; 257];
        let class_length = unsafe { GetClassNameW(hwnd, &mut class_buffer) };
        if class_length <= 0 {
            context.error = Some("GetClassNameW failed during descendant-control discovery".into());
            return BOOL(1);
        }

        let class_name = String::from_utf16_lossy(&class_buffer[..class_length as usize]);
        // SAFETY: selector points to the borrowed selector that remains alive for enumeration.
        let selector = unsafe { &*context.selector };
        if selector.matches(control_id, &class_name) {
            context.matches.push(MatchedControlHandle {
                hwnd,
                control_id,
                class_name,
            });
        }

        BOOL(1)
    }

    let mut context = SearchContext {
        pid,
        selector,
        matches: Vec::new(),
        error: None,
    };
    let context_pointer = &mut context as *mut SearchContext;

    // SAFETY: inspect_control has the required callback ABI. EnumChildWindows is synchronous,
    // parent was returned by EnumWindows, and the context remains valid during enumeration.
    let _ = unsafe {
        EnumChildWindows(
            Some(parent),
            Some(inspect_control),
            LPARAM(context_pointer as isize),
        )
    };

    if let Some(error) = context.error {
        return Err(AppError::runtime(format!(
            "could not inspect descendant controls for companion process {pid}: {error}"
        )));
    }

    Ok(context.matches)
}

#[cfg(windows)]
pub fn find_top_level_window(
    pid: u32,
    matcher: &WindowTitleMatcher,
) -> Result<Option<String>, AppError> {
    Ok(matching_top_level_windows(pid, matcher)?
        .into_iter()
        .next()
        .map(|matched| matched.title))
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
    for window in matching_top_level_windows(pid, window_matcher)? {
        if let Some(control) = matching_descendant_controls(window.hwnd, pid, control_selector)?
            .into_iter()
            .next()
        {
            return Ok(Some(MatchedControl {
                window_title: window.title,
                control_id: control.control_id,
                class_name: control.class_name,
            }));
        }
    }

    Ok(None)
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

#[cfg(windows)]
fn send_bounded_window_message(
    hwnd: windows::Win32::Foundation::HWND,
    message: u32,
    wparam: usize,
    lparam: isize,
    deadline: std::time::Instant,
    operation: &str,
) -> Result<isize, AppError> {
    const SMTO_BLOCK: u32 = 0x0001;
    const SMTO_ABORTIFHUNG: u32 = 0x0002;
    const SMTO_ERRORONEXIT: u32 = 0x0020;

    #[link(name = "user32")]
    unsafe extern "system" {
        fn SendMessageTimeoutW(
            hwnd: *mut core::ffi::c_void,
            message: u32,
            wparam: usize,
            lparam: isize,
            flags: u32,
            timeout_ms: u32,
            result: *mut usize,
        ) -> isize;
    }

    let remaining = deadline.saturating_duration_since(std::time::Instant::now());
    if remaining.is_zero() {
        return Err(AppError::runtime(format!(
            "{operation} could not start because the bounded message deadline expired"
        )));
    }
    let timeout_ms =
        u32::try_from(remaining.as_millis().max(1).min(u128::from(u32::MAX))).unwrap_or(u32::MAX);

    let mut result = 0_usize;
    // SAFETY: hwnd was discovered from the exact launched process and is revalidated by the
    // receiving Win32 call. The output pointer is valid for the duration of the synchronous call.
    let sent = unsafe {
        SendMessageTimeoutW(
            hwnd.0,
            message,
            wparam,
            lparam,
            SMTO_BLOCK | SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
            timeout_ms,
            &mut result,
        )
    };

    if sent == 0 {
        Err(AppError::runtime(format!(
            "{operation} failed or timed out within the bounded message deadline"
        )))
    } else {
        Ok(result as isize)
    }
}

#[cfg(windows)]
pub fn select_combo_box_index(
    pid: u32,
    window_matcher: &WindowTitleMatcher,
    control_selector: &ControlSelector,
    requested_index: u32,
    message_timeout_ms: u32,
) -> Result<ComboBoxSelectionStatus, AppError> {
    use std::time::{Duration, Instant};
    use windows::Win32::UI::WindowsAndMessaging::{
        CB_GETCOUNT, CB_GETCURSEL, CB_SETCURSEL, CBN_SELCHANGE, WM_COMMAND,
    };

    let windows = matching_top_level_windows(pid, window_matcher)?;
    let window = match windows.as_slice() {
        [] => {
            return Ok(ComboBoxSelectionStatus::Pending {
                reason: "matching parent window is not available".into(),
            });
        }
        [window] => window,
        _ => {
            return Err(AppError::runtime(format!(
                "ambiguous parent window selector for companion process {pid}: {} visible top-level windows matched {}",
                windows.len(),
                window_matcher.description()
            )));
        }
    };

    let controls = matching_descendant_controls(window.hwnd, pid, control_selector)?;
    let control = match controls.as_slice() {
        [] => {
            return Ok(ComboBoxSelectionStatus::Pending {
                reason: "matching visible enabled descendant control is not available".into(),
            });
        }
        [control] => control,
        _ => {
            return Err(AppError::runtime(format!(
                "ambiguous control selector in window {:?}: {} visible enabled descendant controls matched {}",
                window.title,
                controls.len(),
                control_selector.description()
            )));
        }
    };

    if control.class_name != "ComboBox" {
        return Err(AppError::runtime(format!(
            "matched control ID {} in window {:?} has unsupported runtime class {:?}; expected exactly \"ComboBox\"",
            control.control_id, window.title, control.class_name
        )));
    }

    let deadline = Instant::now() + Duration::from_millis(u64::from(message_timeout_ms.max(1)));
    let item_count =
        send_bounded_window_message(control.hwnd, CB_GETCOUNT, 0, 0, deadline, "CB_GETCOUNT")?;
    if item_count < 0 {
        return Err(AppError::runtime(format!(
            "CB_GETCOUNT failed for control ID {} in window {:?}",
            control.control_id, window.title
        )));
    }
    if u64::from(requested_index) >= item_count as u64 {
        return Ok(ComboBoxSelectionStatus::Pending {
            reason: format!(
                "requested zero-based index {requested_index} is unavailable; current item count is {item_count}"
            ),
        });
    }

    let prior_raw = send_bounded_window_message(
        control.hwnd,
        CB_GETCURSEL,
        0,
        0,
        deadline,
        "CB_GETCURSEL before selection",
    )?;
    let prior_index = if prior_raw == -1 {
        None
    } else {
        Some(u32::try_from(prior_raw).map_err(|_| {
            AppError::runtime(format!(
                "CB_GETCURSEL returned invalid index {prior_raw} for control ID {} in window {:?}",
                control.control_id, window.title
            ))
        })?)
    };

    if prior_index == Some(requested_index) {
        return Ok(ComboBoxSelectionStatus::Complete(ComboBoxSelection {
            window_title: window.title.clone(),
            control_id: control.control_id,
            class_name: control.class_name.clone(),
            requested_index,
            prior_index,
            resulting_index: requested_index,
            notification_sent: false,
        }));
    }

    let set_result = send_bounded_window_message(
        control.hwnd,
        CB_SETCURSEL,
        requested_index as usize,
        0,
        deadline,
        "CB_SETCURSEL",
    )?;
    if set_result != requested_index as isize {
        return Err(AppError::runtime(format!(
            "CB_SETCURSEL returned {set_result} instead of requested index {requested_index} for control ID {} in window {:?}",
            control.control_id, window.title
        )));
    }

    let selected_before_notification = send_bounded_window_message(
        control.hwnd,
        CB_GETCURSEL,
        0,
        0,
        deadline,
        "CB_GETCURSEL after selection",
    )?;
    if selected_before_notification != requested_index as isize {
        return Err(AppError::runtime(format!(
            "combo-box selection verification failed before notification for control ID {} in window {:?}: requested index {requested_index}, resulting index {selected_before_notification}",
            control.control_id, window.title
        )));
    }

    let notification_wparam =
        (control.control_id as u16 as usize) | ((CBN_SELCHANGE as usize) << 16);
    let _ = send_bounded_window_message(
        window.hwnd,
        WM_COMMAND,
        notification_wparam,
        control.hwnd.0 as isize,
        deadline,
        "WM_COMMAND CBN_SELCHANGE parent notification",
    )?;

    let resulting_raw = send_bounded_window_message(
        control.hwnd,
        CB_GETCURSEL,
        0,
        0,
        deadline,
        "CB_GETCURSEL after parent notification",
    )?;
    if resulting_raw != requested_index as isize {
        return Err(AppError::runtime(format!(
            "combo-box selection verification failed after parent notification for control ID {} in window {:?}: requested index {requested_index}, resulting index {resulting_raw}",
            control.control_id, window.title
        )));
    }

    Ok(ComboBoxSelectionStatus::Complete(ComboBoxSelection {
        window_title: window.title.clone(),
        control_id: control.control_id,
        class_name: control.class_name.clone(),
        requested_index,
        prior_index,
        resulting_index: requested_index,
        notification_sent: true,
    }))
}

#[cfg(not(windows))]
pub fn select_combo_box_index(
    _pid: u32,
    _window_matcher: &WindowTitleMatcher,
    _control_selector: &ControlSelector,
    _requested_index: u32,
    _message_timeout_ms: u32,
) -> Result<ComboBoxSelectionStatus, AppError> {
    Err(AppError::runtime(
        "select-combo-box-index preparation is only available in Windows builds",
    ))
}
