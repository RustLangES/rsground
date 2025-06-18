pub mod error;

use os_pipe::{PipeReader, PipeWriter};
use std::io::Read;
use std::path::{Path, PathBuf};
use tokio::{fs, io};

use hakoniwa::{Child, Command, Container, ExitStatus, Output};

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

    async fn collect_output(cmd: &mut Command) -> Result<Output, hakoniwa::Error> {
        let mut child = cmd
            .envs(BASE_ENV)
            .current_dir("/home")
            .stdin(hakoniwa::Stdio::MakePipe)
            .stdout(hakoniwa::Stdio::MakePipe)
            .stderr(hakoniwa::Stdio::MakePipe)
            .spawn()?;

        let stdout = child.stdout.take();
        let stdout = tokio::spawn(async {
            let mut buf = Vec::new();

            let Some(mut stdout) = stdout else { return buf };

            _ = stdout.read_to_end(&mut buf);

            buf
        });
        let stderr = child.stderr.take();
        let stderr = tokio::spawn(async {
            let mut buf = Vec::new();

            let Some(mut stderr) = stderr else { return buf };

            _ = stderr.read_to_end(&mut buf);

            buf
        });

        let status = tokio::spawn(async move { child.wait() });

        let (status, stdout, stderr) = tokio::join!(status, stdout, stderr);

        let status = status
            .inspect_err(|err| eprintln!("Join error: {err}"))
            .map(|o| o.inspect_err(|err| eprintln!("Join error: {err}")).ok())
            .ok()
            .flatten()
            .unwrap_or(ExitStatus {
                code: 126,
                reason: "Cannot retrieve exit status".to_owned(),
                exit_code: None,
                rusage: None,
            });

        let stdout = stdout
            .inspect_err(|err| eprintln!("Join error: {err}"))
            .unwrap_or_default();

        let stderr = stderr
            .inspect_err(|err| eprintln!("Join error: {err}"))
            .unwrap_or_default();

        Ok(Output {
            status,
            stdout,
            stderr,
        })
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

    pub async fn run(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Output, hakoniwa::Error> {
        Self::collect_output(self.container.command(cmd.as_ref()).args(args)).await
    }

    pub async fn run_bash(
        &self,
        cmd: impl AsRef<str>,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Output, hakoniwa::Error> {
        let args = args
            .into_iter()
            .map(|a| a.as_ref().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let cmd = format!("{} {args}", cmd.as_ref());
        Self::collect_output(self.container.command("/bin/bash").arg("-c").arg(&cmd)).await
    }

    pub async fn run_rustc(
        &self,
        args: impl IntoIterator<Item = impl AsRef<str>>,
    ) -> Result<Output, hakoniwa::Error> {
        Self::collect_output(
            self.container
                .command("/bin/rustc")
                // -C linker=/bin/ld -C link-args=-L/lib -C link-args=-L/lib/gcc/x86_64-unknown-linux-gnu/14.2.1
                .args(args),
        )
        .await
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
