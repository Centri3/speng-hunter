use {
    bytemuck::Pod,
    std::{
        env, ffi::OsString, fs::File, io::Write, mem, os::windows::ffi::OsStringExt, path::PathBuf,
        process::Command, sync::OnceLock, thread, time::Duration,
    },
    sysinfo::{PidExt, ProcessExt, System, SystemExt},
    windows::Win32::{
        Foundation::{GetLastError, BOOL, HANDLE, HWND, LPARAM, WPARAM},
        System::{Diagnostics::Debug, ProcessStatus, Threading},
        UI::WindowsAndMessaging::{
            EnumWindows, GetWindowTextLengthW, GetWindowTextW, SendMessageW, WM_LBUTTONDOWN,
            WM_LBUTTONUP, WNDENUMPROC,
        },
    },
};

pub struct Handler {
    h: HANDLE,
    hwnd: HWND,
    pub exe: PathBuf,
}

impl Handler {
    pub fn new() -> Self {
        // I don't feel like making the code correct, so this works for the time being
        let mut sys = System::new_all();
        sys.refresh_all();

        let process = sys
            .processes_by_exact_name("SpaceEngine.exe")
            .next()
            .expect("SpaceEngine.exe is not open!");

        let handle = unsafe {
            Threading::OpenProcess(Threading::PROCESS_ALL_ACCESS, false, process.pid().as_u32())
        }
        .expect("failed to open handle to SpaceEngine.exe");

        static WINDOW_HWND: OnceLock<HWND> = OnceLock::new();

        unsafe extern "system" fn enum_window(hwnd: HWND, _: LPARAM) -> BOOL {
            let length = GetWindowTextLengthW(hwnd);
            let mut bytes = vec![0u16; length as usize];

            GetWindowTextW(hwnd, &mut bytes);

            if OsString::from_wide(&bytes)
                .into_string()
                .unwrap()
                .contains("SpaceEngine")
            {
                WINDOW_HWND.set(hwnd);
            }

            return BOOL::from(true);
        }

        unsafe {
            EnumWindows(Some(enum_window), LPARAM(0isize)).unwrap();
        }

        Self {
            h: handle,
            hwnd: *WINDOW_HWND.get().unwrap(),
            exe: process.exe().to_path_buf(),
        }
    }

    pub fn base(&self) -> usize {
        let base = [0usize];

        unsafe {
            ProcessStatus::K32EnumProcessModules(
                self.h,
                base.as_ptr() as _,
                mem::size_of_val(&base) as _,
                &mut 0u32,
            );
        }

        base[0usize]
    }

    pub fn read_bytes(&self, base: usize, size: usize) -> Vec<u8> {
        let bytes = vec![0u8; size];

        unsafe {
            Debug::ReadProcessMemory(self.h, base as _, bytes.as_ptr() as _, bytes.len(), None);
        }

        bytes.to_vec()
    }

    /// Convenience function to call `read_bytes` with any type implementing
    /// `Pod`, rather than `Vec<u8>`.
    pub fn read<T: Pod>(&self, base: usize) -> T {
        *bytemuck::from_bytes::<T>(&self.read_bytes(base, mem::size_of::<T>()))
    }

    pub fn write_bytes(&self, bytes: &[u8], base: usize) {
        unsafe {
            Debug::WriteProcessMemory(self.h, base as _, bytes.as_ptr() as _, bytes.len(), None);
        }
    }

    /// Convenience function to call `write_bytes` with any type implementing
    /// `Pod`, rather than `&[u8]`.
    pub fn write<T: Pod>(&self, buffer: T, base: usize) {
        self.write_bytes(bytemuck::bytes_of(&buffer), base);
    }

    /// Create and run an SE script.
    pub fn run_script(&self, name: &str, buffer: &[u8]) {
        let mut file =
            File::create(name).unwrap_or_else(|_| panic!("failed to create script `{}`", name));
        let mut path = env::current_dir().unwrap();
        path.push(name);

        file.write_all(buffer)
            .unwrap_or_else(|_| panic!("failed to write to script `{}`", name));

        Command::new(self.exe.clone()).arg(path).spawn().unwrap();
    }

    /// Click by sending a message.
    pub fn click(&self, x: i32, y: i32) {
        unsafe {
            SendMessageW(
                self.hwnd,
                WM_LBUTTONDOWN,
                WPARAM(0usize),
                LPARAM(isize::overflowing_shl(y as isize, 16).0 | (x & 0xFFFF) as isize),
            )
        };

        unsafe {
            SendMessageW(
                self.hwnd,
                WM_LBUTTONUP,
                WPARAM(0usize),
                LPARAM(isize::overflowing_shl(y as isize, 16).0 | (x & 0xFFFF) as isize),
            )
        };
    }
}
