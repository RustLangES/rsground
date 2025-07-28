#![allow(async_fn_in_trait)]

pub mod error;
pub mod futures_ext;
pub mod hakoniwa_ext;

use futures_ext::{FutureExt, FutureOption, OptionalFuture};
use hakoniwa::{Child, Command, Container, ExitStatus, Output};
use hakoniwa_ext::{AsyncOsReader, HakoniwaChildExt};
pub use os_pipe::{PipeReader, PipeWriter};
use std::future::Future;
use std::path::{Path, PathBuf};
use tokio::sync::oneshot;
use tokio::{fs, io};

pub const BASE_ENV: [(&str, &str); 3] = [
    ("HOME", "/home"),
    ("PATH", "/bin"),
    ("LD_LIBRARY_PATH", "/lib:/lib64:/libexec"),
];

pub struct Runner {
    container: Container,
    temp_home: PathBuf,
}

impl Runner {
    async fn create_home() -> PathBuf {
        let mut temp_home = PathBuf::from("/tmp");
        temp_home.push(uuid::Uuid::new_v4().simple().to_string());

        fs::create_dir(&temp_home)
            .await
            .expect("Cannot create home");

        temp_home
    }

    fn create_container(temp_home: &str) -> Container {
        Container::new()
            .hostname("rsground")
            .rootfs(concat!(env!("CARGO_MANIFEST_DIR"), "/lxc_rootfs"))
            .tmpfsmount("/tmp")
            .devfsmount("/dev")
            .procfsmount("/proc")
            .uidmap(1001)
            .gidmap(100)
            .bindmount_rw(temp_home, "/home")
            // FIXME: This needs to set resource limit
            // .setrlimit(hakoniwa::Rlimit::*, soft_limit, hard_limit)
            .clone()
    }

    pub async fn new() -> Result<Self, ()> {
        let temp_home = Self::create_home().await;
        let container = Self::create_container(temp_home.to_str().unwrap());

        Ok(Self {
            container,
            temp_home,
        })
    }

    pub async fn create_file(
        &self,
        container_path: impl AsRef<str>,
        content: impl AsRef<str>,
    ) -> io::Result<()> {
        let mut home = self.temp_home.clone();
        // FIXME: This is security breach, needs to check if path is inside the container
        home.push(container_path.as_ref());

        fs::create_dir_all(home.parent().unwrap()).await.unwrap();

        fs::write(home, content.as_ref()).await
    }

    pub async fn copy_file_from_runner(
        &self,
        other: &Runner,
        host_path: impl AsRef<Path>,
        other_path: impl AsRef<Path>,
    ) {
        let mut host_file_path = self.temp_home.clone();
        host_file_path.push(host_path);

        let mut other_file_path = other.temp_home.clone();
        other_file_path.push(other_path);

        fs::copy(other_file_path, host_file_path)
            .await
            .expect("Skill issuer de manual");
    }

    pub async fn collect_output(cmd: &mut Command) -> Result<Output, hakoniwa::Error> {
        async fn collect(mut stream: AsyncOsReader) -> Vec<u8> {
            let mut buf = Vec::new();

            let bytes = stream.read_to_end(&mut buf).await;

            _ = dbg!(bytes);

            buf
        }

        Self::stream_output(cmd, collect, collect, None)
            .await
            .map(|(status, stdout, stderr)| Output {
                status,
                stdout,
                stderr,
            })
    }

    pub async fn stream_output<Stdout, Stderr, StdoutAsync, StderrAsync, StdoutFn, StderrFn>(
        cmd: &mut Command,
        stdout_fn: StdoutFn,
        stderr_fn: StderrFn,
        abort: Option<oneshot::Receiver<()>>,
    ) -> Result<(ExitStatus, Stdout, Stderr), hakoniwa::Error>
    where
        Stdout: Default + Send + 'static,
        StdoutAsync: Future<Output = Stdout> + Send + 'static,
        StdoutFn: Send + 'static + FnOnce(AsyncOsReader) -> StdoutAsync,
        Stderr: Default + Send + 'static,
        StderrAsync: Future<Output = Stderr> + Send + 'static,
        StderrFn: Send + 'static + FnOnce(AsyncOsReader) -> StderrAsync,
    {
        let mut child = cmd
            .envs(BASE_ENV)
            .current_dir("/home")
            .stdin(hakoniwa::Stdio::MakePipe)
            .stdout(hakoniwa::Stdio::MakePipe)
            .stderr(hakoniwa::Stdio::MakePipe)
            .spawn()?;

        let stdout = child
            .stdout
            .take()
            .map(AsyncOsReader::from)
            .map(stdout_fn)
            .as_fut()
            .or_default()
            .spawn();

        let stderr = child
            .stderr
            .take()
            .map(AsyncOsReader::from)
            .map(stderr_fn)
            .as_fut()
            .or_default()
            .spawn();

        let status = if let Some(abort) = abort {
            tokio::spawn(async move { child.wait_or_abort(abort).wrap_fut(Ok).await })
        } else {
            tokio::task::spawn_blocking(move || child.wait())
        };

        let (status, stdout, stderr) =
            tokio::try_join!(status, stdout, stderr).map_err(|join_error| {
                return hakoniwa::Error::Unexpected(format!("Join error: {join_error}"));
            })?;

        let status = status
            .inspect_err(|err| eprintln!("Join error: {err}"))
            .unwrap_or(ExitStatus {
                code: 126,
                reason: "Cannot retrieve exit status".to_owned(),
                exit_code: None,
                rusage: None,
            });

        Ok((status, stdout, stderr))
    }

    /// Spawn process with shared stdio.
    /// Focused in interactive shell for manual testing
    pub fn spawn(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<hakoniwa::Child, hakoniwa::Error> {
        self.container
            .command(cmd.as_ref())
            .args(args)
            .envs(BASE_ENV)
            .stdin(hakoniwa::Stdio::Inherit)
            .stdout(hakoniwa::Stdio::Inherit)
            .stderr(hakoniwa::Stdio::Inherit)
            .current_dir("/home")
            .spawn()
    }

    pub fn start_rls(&mut self) -> hakoniwa::Result<(Child, PipeWriter, PipeReader, PipeReader)> {
        let mut child = self
            .container
            .command("/bin/rust-analyzer")
            .envs(BASE_ENV)
            .stdin(hakoniwa::Stdio::MakePipe)
            .stdout(hakoniwa::Stdio::MakePipe)
            .stderr(hakoniwa::Stdio::MakePipe)
            .current_dir("/home")
            .spawn()?;

        let stdin = child.stdin.take().expect("Needs communication >:(");
        let stdout = child.stdout.take().expect("Needs communication >:(");
        let stderr = child.stderr.take().expect("Needs communication >:(");

        Ok((child, stdin, stdout, stderr))
    }

    pub fn cmd(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Command {
        let mut cmd = self.container.command(cmd.as_ref());
        cmd.args(args);
        cmd
    }

    pub fn cmd_bash(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Command {
        let args = args
            .into_iter()
            .map(|a| format!("{:?}", a.as_ref()))
            .collect::<Vec<_>>()
            .join(" ");
        let arg = format!("{} {args}", cmd.as_ref());

        let mut cmd = self.container.command("/bin/bash");
        cmd.arg("-c").arg(&arg);
        cmd
    }

    pub fn cmd_rustc(&self, args: impl IntoIterator<Item = impl AsRef<str>>) -> Command {
        let mut cmd = self.container.command("/bin/rustc");
        // -C linker=/bin/ld -C link-args=-L/lib -C link-args=-L/lib/gcc/x86_64-unknown-linux-gnu/14.2.1
        cmd.args(args);
        cmd
    }

    pub async fn patch_binary(&self, path: impl AsRef<str>) -> Result<Output, hakoniwa::Error> {
        Self::collect_output(
            self.container
                .command("/bin/patchelf")
                .arg("--set-interpreter")
                .arg("/lib/ld-linux-x86-64.so.2")
                .arg(path.as_ref()),
        )
        .await
    }
}

impl Drop for Runner {
    fn drop(&mut self) {
        _ = std::fs::remove_dir_all(&self.temp_home)
            .inspect_err(|err| eprintln!("cannot delete temp home: {err}"));
    }
}
