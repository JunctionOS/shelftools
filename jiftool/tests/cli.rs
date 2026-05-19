use std::process::{Command, Output};

fn jiftool(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_jiftool"))
        .args(args)
        .output()
        .expect("failed to run jiftool")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn top_level_help_lists_unified_commands() {
    let output = jiftool(&["--help"]);

    assert!(output.status.success(), "stderr:\n{}", stderr(&output));

    let stdout = stdout(&output);
    for command in ["read", "check", "modify", "trace", "compare", "time"] {
        assert!(
            stdout.contains(command),
            "top-level help did not mention `{command}`:\n{stdout}"
        );
    }
}

#[test]
fn read_help_lists_inspection_modes() {
    let output = jiftool(&["read", "--help"]);

    assert!(output.status.success(), "stderr:\n{}", stderr(&output));

    let stdout = stdout(&output);
    for command in ["summary", "pheaders", "ord", "raw"] {
        assert!(
            stdout.contains(command),
            "read help did not mention `{command}`:\n{stdout}"
        );
    }
}

#[test]
fn modify_help_lists_mutating_operations() {
    let output = jiftool(&["modify", "--help"]);

    assert!(output.status.success(), "stderr:\n{}", stderr(&output));

    let stdout = stdout(&output);
    for command in [
        "rewrite",
        "rename",
        "build-itrees",
        "fragment-vmas",
        "setup-prefetch",
        "tag-vmas",
        "add-ord",
    ] {
        assert!(
            stdout.contains(command),
            "modify help did not mention `{command}`:\n{stdout}"
        );
    }
}

#[test]
fn missing_jif_errors_include_path_context() {
    let output = jiftool(&["check", "missing-regression.jif"]);

    assert!(
        !output.status.success(),
        "missing file check unexpectedly succeeded"
    );

    let stderr = stderr(&output);
    assert!(
        stderr.contains("failed to open JIF missing-regression.jif"),
        "missing file error lacked path context:\n{stderr}"
    );
}
