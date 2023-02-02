use anyhow::{Context, Result};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    // println!("args: {:?}", args);
    // println!("command: {}", command);
    let command_args = &args[4..];
    let mut is_stderr = false;
    if command_args[0] == "echo_stderr" {
        is_stderr = true;
    }
    // println!("is_stderr: {}", is_stderr);
    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        })?;
    
    if output.status.success() {
        let target = if is_stderr { &output.stderr } else { &output.stdout };
        let stdio = std::str::from_utf8(target)?;
        if is_stderr {
            eprintln!("{}", stdio);
        } else {
            println!("{}", stdio);
        }
    } else {
        std::process::exit(1);
    }

    Ok(())
}
