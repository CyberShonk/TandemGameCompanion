use crate::error::AppError;

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
