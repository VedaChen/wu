use std::sync::{Arc, Mutex, PoisonError};
use std::time::Duration;

use tokio::io::{AsyncRead, AsyncReadExt as _};

use crate::CancellationToken;
use crate::process::{Command, Stdio};

const KEEP_BYTES: usize = 1024 * 1024;
// Claude Code's Bash tool hands the model at most this many characters.
const MODEL_OUTPUT_CHARS: usize = 30_000;
const PIPE_SETTLE: Duration = Duration::from_secs(1);

pub(super) struct ShellRun {
    stdout: String,
    stderr: String,
    pub(super) failed: bool,
}

impl ShellRun {
    pub(super) fn spawn_failed(error: &std::io::Error) -> Self {
        Self {
            stdout: String::new(),
            stderr: format!("Command failed: {error}"),
            failed: true,
        }
    }

    pub(super) fn display(&self) -> Option<String> {
        let shown = [self.stdout.as_str(), self.stderr.as_str()]
            .into_iter()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        (!shown.is_empty()).then_some(shown)
    }

    pub(super) fn for_model(&self) -> String {
        format!(
            "<bash-stdout>{}</bash-stdout><bash-stderr>{}</bash-stderr>",
            clip_middle(&self.stdout, MODEL_OUTPUT_CHARS),
            clip_middle(&self.stderr, MODEL_OUTPUT_CHARS)
        )
    }
}

fn shell_process(command: &str) -> Command {
    #[cfg(unix)]
    {
        let shell = std::env::var_os("SHELL")
            .filter(|shell| !shell.is_empty())
            .unwrap_or_else(|| "/bin/sh".into());
        let mut process = Command::new(shell);
        process.arg("-lc").arg(command).process_group(0);
        process
    }
    #[cfg(not(unix))]
    {
        let mut process = Command::new("cmd");
        process.arg("/C").arg(command);
        process
    }
}

fn drain(
    pipe: Option<impl AsyncRead + Unpin + Send + 'static>,
) -> (Arc<Mutex<Vec<u8>>>, tokio::task::JoinHandle<()>) {
    let kept = Arc::new(Mutex::new(Vec::new()));
    let task = tokio::spawn({
        let kept = kept.clone();
        async move {
            let Some(mut pipe) = pipe else {
                return;
            };
            let mut chunk = [0_u8; 8192];
            loop {
                match pipe.read(&mut chunk).await {
                    Ok(0) => return,
                    Ok(read) => {
                        let mut kept = kept.lock().unwrap_or_else(PoisonError::into_inner);
                        let room = KEEP_BYTES.saturating_sub(kept.len());
                        kept.extend_from_slice(&chunk[..read.min(room)]);
                    }
                    Err(error) => {
                        log::debug!("couldn't read a shell command's output: {error}");
                        return;
                    }
                }
            }
        }
    });
    (kept, task)
}

fn collected(kept: &Mutex<Vec<u8>>) -> String {
    let bytes = kept.lock().unwrap_or_else(PoisonError::into_inner);
    String::from_utf8_lossy(&bytes).trim_end().to_owned()
}

/// `Ok(None)` when the user interrupted the command.
pub(super) async fn run(
    command: &str,
    cwd: &std::path::Path,
    interrupt: &CancellationToken,
    kill_grace: Duration,
) -> std::io::Result<Option<ShellRun>> {
    let mut process = shell_process(command);
    process
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = process.spawn()?;
    let (stdout, stdout_task) = drain(child.stdout.take());
    let (stderr, stderr_task) = drain(child.stderr.take());
    let status = tokio::select! {
        status = child.wait() => Some(status?),
        () = interrupt.cancelled() => None,
    };
    let Some(status) = status else {
        crate::shutdown_child(&mut child, kill_grace).await;
        stdout_task.abort();
        stderr_task.abort();
        return Ok(None);
    };
    // A command that leaves a background job running keeps the pipes open after it exits.
    let settled = tokio::time::timeout(PIPE_SETTLE, async {
        stdout_task.await.ok();
        stderr_task.await.ok();
    })
    .await;
    if settled.is_err() {
        log::debug!("shell command's output pipes stayed open after it exited");
    }
    let mut stderr = collected(&stderr);
    if !status.success() {
        let exit = crate::describe_exit(Some(status));
        if !stderr.is_empty() {
            stderr.push('\n');
        }
        let mut exit = exit.chars();
        stderr.extend(exit.next().into_iter().flat_map(char::to_uppercase));
        stderr.push_str(exit.as_str());
    }
    Ok(Some(ShellRun {
        stdout: collected(&stdout),
        stderr,
        failed: !status.success(),
    }))
}

fn clip_middle(text: &str, limit: usize) -> String {
    let total = text.chars().count();
    if total <= limit {
        return text.to_owned();
    }
    let half = limit / 2;
    let head: String = text.chars().take(half).collect();
    let tail: String = text.chars().skip(total - half).collect();
    format!(
        "{head}\n\n… {} characters cut …\n\n{tail}",
        total - half * 2
    )
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    #[tokio::test]
    async fn runs_in_the_folder_and_keeps_both_streams() {
        let folder = tempfile::tempdir().unwrap();
        let run = run(
            "pwd -P; echo oops 1>&2; exit 3",
            folder.path(),
            &CancellationToken::new(),
            Duration::from_secs(1),
        )
        .await
        .unwrap()
        .unwrap();
        assert!(run.failed);
        let shown = run.display().unwrap();
        let folder = folder.path().canonicalize().unwrap();
        assert!(shown.contains(&*folder.to_string_lossy()), "{shown}");
        assert!(shown.ends_with("oops\nExit code 3"), "{shown}");
        assert!(
            run.for_model()
                .contains("<bash-stderr>oops\nExit code 3</bash-stderr>")
        );
    }

    #[tokio::test]
    async fn an_interrupt_stops_the_command() {
        let interrupt = CancellationToken::new();
        let started = std::time::Instant::now();
        let folder = std::env::temp_dir();
        let running = run("sleep 30", &folder, &interrupt, Duration::from_millis(200));
        interrupt.cancel();
        assert!(running.await.unwrap().is_none());
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn long_output_keeps_its_start_and_end() {
        let text = format!("{}{}", "a".repeat(20), "b".repeat(20));
        assert_eq!(
            clip_middle(&text, 10),
            "aaaaa\n\n… 30 characters cut …\n\nbbbbb"
        );
        assert_eq!(clip_middle("short", 10), "short");
    }
}
