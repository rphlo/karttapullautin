// use cargo test -- --ignored --nocapture to run this test

#[test]
#[ignore]
fn compare_pullauta_to_latest_release() {
    let command = std::process::Command::new("sh")
        .arg("tests/regression.sh")
        .output()
        .expect("failed to execute process");
    if command.status.success() {
        println!(
            "Script output:\n{}",
            String::from_utf8_lossy(&command.stdout)
        );
    } else {
        eprintln!(
            "Script failed with stderr:\n{}",
            String::from_utf8_lossy(&command.stderr)
        );
    }
}
