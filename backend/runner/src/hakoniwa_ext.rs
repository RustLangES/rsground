use std::future::Future;
use std::io;
use std::task::{ready, Poll};
use std::time::Duration;
use std::{io::Read, ops, os::fd::AsFd};

use async_io::Async;
use futures::Stream;
use hakoniwa::{Child, ExitStatus};
use nix::libc::pid_t;
use nix::sys::signal::{self, Signal};
use nix::{
    sys::wait::{self, WaitPidFlag, WaitStatus},
    unistd::Pid,
};
use tokio::io::AsyncRead;

pub trait HakoniwaChildExt {
    fn try_wait(&self) -> Option<ExitStatus>;
    async fn async_wait(&self) -> ExitStatus;
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

    async fn async_wait(&self) -> ExitStatus {
        let mut status_check_interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            status_check_interval.tick().await;

            if let Some(status) = self.try_wait() {
                return status;
            }
        }
    }

    async fn wait_or_abort<A: Future + Unpin>(&self, mut abort: A) -> ExitStatus {
        let child_pid = Pid::from_raw(self.id() as pid_t);
        let waiter = self.async_wait();

        loop {
            tokio::select! {
                status = waiter => {
                    return status
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

impl AsyncOsReader {
    /// N is the buffer size for reads
    pub fn stream<const N: usize>(self) -> AsyncOsReaderStream<N> {
        AsyncOsReaderStream { inner: self }
    }

    pub fn into_lsp(self) -> LspStdoutReader {
        LspStdoutReader { inner: self.0 }
    }
}

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

impl ops::Deref for AsyncOsReader {
    type Target = os_pipe::PipeReader;

    fn deref(&self) -> &Self::Target {
        &self.0.get_ref()
    }
}

impl AsyncRead for AsyncOsReader {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        ready!(self.0.poll_readable(cx))?;

        unsafe { self.0.get_mut() }.read(buf.initialize_unfilled())?;

        Poll::Ready(Ok(()))
    }
}

pub struct AsyncOsReaderStream<const N: usize> {
    inner: AsyncOsReader,
}

impl<const N: usize> Stream for AsyncOsReaderStream<N> {
    type Item = Result<Vec<u8>, io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        ready!(self.inner.0.poll_readable(cx))?;

        let buf = &mut [0; N];

        let readed = unsafe { self.inner.0.get_mut() }.read(buf)?;

        if readed == 0 {
            Poll::Ready(None)
        } else {
            Poll::Ready(Some(Ok(buf[..readed].to_vec())))
        }
    }
}

pub struct LspStdoutReader {
    inner: Async<os_pipe::PipeReader>,
}

impl LspStdoutReader {
    fn inner(&self) -> &Async<os_pipe::PipeReader> {
        &self.inner
    }
}

impl From<os_pipe::PipeReader> for LspStdoutReader {
    fn from(value: os_pipe::PipeReader) -> Self {
        Self {
            inner: Async::new(value).expect("Cannot create async wrapper"),
        }
    }
}

impl AsFd for LspStdoutReader {
    fn as_fd(&self) -> std::os::unix::prelude::BorrowedFd<'_> {
        self.inner().as_fd()
    }
}

impl ops::Deref for LspStdoutReader {
    type Target = os_pipe::PipeReader;

    fn deref(&self) -> &Self::Target {
        &self.inner().get_ref()
    }
}

impl Stream for LspStdoutReader {
    type Item = Result<String, io::Error>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        ready!(self.inner().poll_readable(cx))?;

        let reader = unsafe { self.inner.get_mut() };

        const CONTENT_LENGTH: &[u8] = b"Content-Length: ";

        let buf = &mut [0; CONTENT_LENGTH.len()];
        let readed = reader.read(buf)?;

        if readed == 0 {
            return Poll::Ready(None);
        }

        assert_eq!(buf, CONTENT_LENGTH);

        let buf = &mut [0; 1];
        let mut content_length = 0usize;

        loop {
            reader.read_exact(buf)?;

            match buf[0] {
                b'0'..=b'9' => {
                    let digit = buf[0] - b'0';
                    content_length *= 10;
                    content_length += digit as usize;
                }
                b'\r' => {
                    // Collect newline
                    reader.read_exact(buf)?;
                    break;
                }
                _ => {
                    return Poll::Ready(Some(Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Unexpected char",
                    ))));
                }
            }
        }

        // Collect separator between headers and content
        // \r\n
        reader.read_exact(buf)?;
        reader.read_exact(buf)?;

        let mut buf = vec![0; content_length];
        let readed = unsafe { self.inner.get_mut() }.read(&mut buf)?;

        if readed == 0 {
            return Poll::Ready(None);
        }

        let content =
            String::from_utf8(buf).map_err(|err| io::Error::new(io::ErrorKind::InvalidData, err));

        Poll::Ready(Some(content))
    }
}
