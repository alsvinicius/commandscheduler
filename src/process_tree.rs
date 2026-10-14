//! Own the entire command lifetime, including descendants holding inherited pipes.

use std::io;
use tokio::process::{Child, Command};

pub struct ProcessTree {
    #[cfg(unix)]
    pid: u32,
    #[cfg(windows)]
    job: std::os::windows::io::OwnedHandle,
}

impl ProcessTree {
    pub fn spawn(command: &mut Command) -> io::Result<(Child, Self)> {
        #[cfg(unix)]
        command.process_group(0);
        // Assign the child to its job before any user code can create descendants.
        #[cfg(windows)]
        command.creation_flags(windows_sys::Win32::System::Threading::CREATE_SUSPENDED);
        #[allow(unused_mut)]
        let mut child = command.spawn()?;
        #[cfg(unix)]
        let tree = Self {
            pid: child.id().expect("new child has a process id"),
        };
        #[cfg(windows)]
        let tree = match Self::attach_and_resume(&child) {
            Ok(tree) => tree,
            Err(error) => {
                let _ = child.start_kill();
                return Err(error);
            }
        };
        Ok((child, tree))
    }

    pub fn terminate(&self) {
        #[cfg(unix)]
        // SAFETY: the PID identifies the separate process group created above.
        unsafe {
            libc::kill(-(self.pid as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            // SAFETY: the job handle is owned and remains open for this call.
            unsafe {
                windows_sys::Win32::System::JobObjects::TerminateJobObject(
                    self.job.as_raw_handle(),
                    1,
                );
            }
        }
    }

    #[cfg(windows)]
    fn attach_and_resume(child: &Child) -> io::Result<Self> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
        use windows_sys::Win32::{
            Foundation::INVALID_HANDLE_VALUE,
            System::{
                Diagnostics::ToolHelp::{
                    CreateToolhelp32Snapshot, TH32CS_SNAPTHREAD, THREADENTRY32, Thread32First,
                    Thread32Next,
                },
                JobObjects::{
                    AssignProcessToJobObject, CreateJobObjectW, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                    JOBOBJECT_BASIC_LIMIT_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
                    JobObjectExtendedLimitInformation, SetInformationJobObject,
                },
                Threading::{OpenThread, ResumeThread, THREAD_SUSPEND_RESUME},
            },
        };
        // SAFETY: all returned handles are checked, owned and closed by OwnedHandle.
        // The child was created suspended and has a live process handle.
        unsafe {
            let raw_job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
            if raw_job.is_null() {
                return Err(io::Error::last_os_error());
            }
            let job = OwnedHandle::from_raw_handle(raw_job);
            let limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION {
                BasicLimitInformation: JOBOBJECT_BASIC_LIMIT_INFORMATION {
                    LimitFlags: JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
                    ..Default::default()
                },
                ..Default::default()
            };
            if SetInformationJobObject(
                job.as_raw_handle(),
                JobObjectExtendedLimitInformation,
                &limits as *const _ as _,
                std::mem::size_of_val(&limits) as u32,
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            if AssignProcessToJobObject(
                job.as_raw_handle(),
                child.raw_handle().expect("new child has a process handle"),
            ) == 0
            {
                return Err(io::Error::last_os_error());
            }
            let tree = Self { job };
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err(io::Error::last_os_error());
            }
            let snapshot = OwnedHandle::from_raw_handle(snapshot);
            let mut thread = THREADENTRY32 {
                dwSize: std::mem::size_of::<THREADENTRY32>() as u32,
                ..Default::default()
            };
            let mut found = Thread32First(snapshot.as_raw_handle(), &mut thread);
            while found != 0 {
                if Some(thread.th32OwnerProcessID) == child.id() {
                    let handle = OpenThread(THREAD_SUSPEND_RESUME, 0, thread.th32ThreadID);
                    if handle.is_null() {
                        return Err(io::Error::last_os_error());
                    }
                    let handle = OwnedHandle::from_raw_handle(handle);
                    if ResumeThread(handle.as_raw_handle()) == u32::MAX {
                        return Err(io::Error::last_os_error());
                    }
                    return Ok(tree);
                }
                found = Thread32Next(snapshot.as_raw_handle(), &mut thread);
            }
            Err(io::Error::other(
                "could not find the suspended child thread",
            ))
        }
    }
}

impl Drop for ProcessTree {
    fn drop(&mut self) {
        self.terminate();
    }
}
