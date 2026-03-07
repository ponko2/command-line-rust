use assert_cmd::cargo::cargo_bin_cmd;
use pretty_assertions::assert_eq;

#[test]
fn runs() {
    let mut cmd = cargo_bin_cmd!();
    let output = cmd.output().expect("fail");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("invalid UTF-8");
    assert_eq!(stdout, "Hello, world!\n");
}

#[test]
fn true_ok() {
    let mut cmd = cargo_bin_cmd!("true");
    cmd.assert().success();
}

#[test]
fn false_not_ok() {
    let mut cmd = cargo_bin_cmd!("false");
    cmd.assert().failure();
}
