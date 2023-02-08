use anyhow::{Context, Result};
use std::ffi::CString;
use libc::{chroot, c_char};
use std::time::{SystemTime, UNIX_EPOCH};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];
    let mut is_stderr = false;
    if command_args[0] == "echo_stderr" {
        is_stderr = true;
    }

    // Filesystem isolation
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    unsafe {
        // https://yoshitsugu.net/posts/2018-03-22-jailing-in-rust.html
        chroot(CString::new(format!("/sandbox{}", nanos).as_bytes())
            .expect("Error in construct CString")
            .as_bytes_with_nul()
            .as_ptr() as *const c_char);
    }
    assert!(std::env::set_current_dir("/").is_ok());
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
                    Some(code) => {
                        let stdout = std::str::from_utf8(&out.stdout)?;
                        print!("{}", stdout);
                        std::process::exit(code);
                    },
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
