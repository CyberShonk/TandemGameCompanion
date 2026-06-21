use crate::error::AppError;

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
