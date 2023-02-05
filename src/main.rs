use anyhow::{Context, Result};
use std::ffi::CString;
use libc::{chroot, c_char};

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

    // Filesystem isolation
    unsafe {
        // https://yoshitsugu.net/posts/2018-03-22-jailing-in-rust.html
        chroot(CString::new("/sandbox".as_bytes())
            .expect("Error in construct CString")
            .as_bytes_with_nul()
            .as_ptr() as *const c_char);
    }
    std::env::set_current_dir("/")?;
    // Workaround: Command::output() expects /dev/null to be present.
    std::fs::create_dir_all("/dev/null")?;

    let output = std::process::Command::new(command)
        .args(command_args)
        .output()
        .with_context(|| {
            format!(
                "Tried to run '{}' with arguments {:?}",
                command, command_args
            )
        });
    match output {
        Ok(out) => {
            if out.status.success() {
                let target = if is_stderr { &out.stderr } else { &out.stdout };
                let stdio = std::str::from_utf8(target)?;
                if is_stderr {
                    eprint!("{}", stdio);
                } else {
                    print!("{}", stdio);
                }
            } else {
                match out.status.code() {
                    Some(code) => std::process::exit(code),
                    None => println!("Process terminated by signal"),
                }
            }
        },
        Err(e) => {
            std::process::exit(e.source().unwrap()
                .downcast_ref::<std::io::Error>()
                .unwrap().raw_os_error().unwrap());
        },
    }

    Ok(())
}
