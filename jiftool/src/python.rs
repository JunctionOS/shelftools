use anyhow::Context;
use std::io::Write;
use std::process::{Command, Stdio};

pub(crate) fn run_python<F>(
    script: &str,
    args: &[String],
    dependency_hint: &str,
    write_input: F,
) -> anyhow::Result<String>
where
    F: FnOnce(&mut dyn Write) -> anyhow::Result<()>,
{
    let mut child = Command::new("python3")
        .arg("-c")
        .arg(script)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .with_context(|| {
            format!("failed to spawn python plotter: make sure {dependency_hint} is installed")
        })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .context("failed to open pipe to python plotter")?;
        write_input(&mut stdin)?;
    }

    let output = child
        .wait_with_output()
        .context("failed to execute python plotter")?;

    if !output.status.success() {
        anyhow::bail!(
            "python plotter exited with {}:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    String::from_utf8(output.stdout).context("python plotter produced non-UTF8 output")
}
