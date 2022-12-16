use {
    bytemuck::Pod,
    std::{env, fs::File, io::Write, mem, path::PathBuf, process::Command},
    sysinfo::{PidExt, ProcessExt, System, SystemExt},
    windows::Win32::{
        Foundation::HANDLE,
        System::{Diagnostics::Debug, ProcessStatus, Threading},
    },
};

pub struct Handler {
    inner: HANDLE,
    exe: PathBuf,
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

        Self {
            inner: handle,
            exe: process.exe().to_path_buf(),
        }
    }

    pub fn base(&self) -> usize {
        let base = [0usize];

        unsafe {
            ProcessStatus::K32EnumProcessModules(
                self.inner,
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
            Debug::ReadProcessMemory(
                self.inner,
                base as _,
                bytes.as_ptr() as _,
                bytes.len(),
                None,
            );
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
            Debug::WriteProcessMemory(
                self.inner,
                base as _,
                bytes.as_ptr() as _,
                bytes.len(),
                None,
            );
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
}
