use std::future::Future;
use std::time::Duration;
use std::{io::Read, ops, os::fd::AsFd, ptr::read};

use async_io::Async;
use hakoniwa::{Child, ExitStatus};
use nix::libc::pid_t;
use nix::sys::signal::{self, Signal};
use nix::{
    sys::wait::{self, WaitPidFlag, WaitStatus},
    unistd::Pid,
};

pub trait HakoniwaChildExt {
    fn try_wait(&self) -> Option<ExitStatus>;
    async fn wait_or_abort<A: Future + Unpin>(&self, abort: A) -> ExitStatus;
}

impl HakoniwaChildExt for Child {
    fn try_wait(&self) -> Option<ExitStatus> {
        match wait::waitpid(Pid::from_raw(self.id() as i32), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::StillAlive) => None,
            Ok(WaitStatus::Exited(_, code)) => Some(ExitStatus {
                code,
                // Mario reference
                reason: "Life is good".to_owned(),
                exit_code: None,
                rusage: None,
            }),
            Ok(WaitStatus::Signaled(_, signal, _) | WaitStatus::Stopped(_, signal)) => {
                Some(ExitStatus {
                    code: signal as i32,
                    reason: signal.as_str().to_owned(),
                    exit_code: None,
                    rusage: None,
                })
            }
            Ok(WaitStatus::Continued(_)) => None,
            Ok(_) => None,
            Err(err) => {
                println!("[ERROR] {err}");
                None
            }
        }
    }

    async fn wait_or_abort<A: Future + Unpin>(&self, mut abort: A) -> ExitStatus {
        let mut status_check_interval = tokio::time::interval(Duration::from_millis(100));
        let child_pid = Pid::from_raw(self.id() as pid_t);

        loop {
            tokio::select! {
            _ = status_check_interval.tick() => {
                if let Some(status) = self.try_wait() {
                    return status
                }
            }
                _ = &mut abort => {
                    _ = signal::kill(child_pid, Signal::SIGKILL);
                    return ExitStatus {
                        code: 137,
                        reason: "Aborted".to_owned(),
                        exit_code: None,
                        rusage: None,
                    };
                },
            }
        }
    }
}

pub struct AsyncOsReader(Async<os_pipe::PipeReader>);

impl From<os_pipe::PipeReader> for AsyncOsReader {
    fn from(value: os_pipe::PipeReader) -> Self {
        Self(Async::new(value).expect("Cannot create async wrapper"))
    }
}

impl AsFd for AsyncOsReader {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsyncOsReader {
    pub async fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        _ = self.0.readable().await?;
        unsafe { self.0.get_mut() }.read(buf)
    }

    pub async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> std::io::Result<usize> {
        let mut total_bytes = 0;

        loop {
            let read_bytes = self.read(buf.as_mut_slice()).await?;

            if read_bytes == 0 {
                break;
            }

            total_bytes += read_bytes;
        }

        Ok(total_bytes)
    }
}

impl ops::Deref for AsyncOsReader {
    type Target = os_pipe::PipeReader;

    fn deref(&self) -> &Self::Target {
        &self.0.get_ref()
    }
}
