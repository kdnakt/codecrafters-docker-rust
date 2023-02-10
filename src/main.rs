use anyhow::{Context, Result};
use libc::{chroot, c_char};
use std::ffi::CString;
use std::time::{SystemTime, UNIX_EPOCH};

// Usage: your_docker.sh run <image> <command> <arg1> <arg2> ...
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    let command = &args[3];
    let command_args = &args[4..];

    // Filesystem isolation
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_nanos();
    let dir = format!("/sandbox/{}", nanos);
    std::fs::create_dir_all(&dir).expect("create_dir_all");
    std::fs::copy(command, format!("{}/command", dir)).expect("copy");
    let err = unsafe {
        // https://yoshitsugu.net/posts/2018-03-22-jailing-in-rust.html
        chroot(CString::new(dir.as_bytes())
            .expect("Error in construct CString")
            .as_bytes_with_nul()
            .as_ptr() as *const c_char)
    };
    if err == -1 {
        panic!("chroot failed.");
    }
    std::fs::create_dir_all("/dev/null").expect("create_dir_all");
    let output = std::process::Command::new("/command")
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
                eprint!("{}", std::str::from_utf8(&out.stderr).unwrap());
                print!("{}", std::str::from_utf8(&out.stdout).unwrap());
            } else {
                match out.status.code() {
                    Some(code) => {
                        let stdout = std::str::from_utf8(&out.stdout)?;
                        print!("{}", stdout);
                        std::process::exit(code);
                    },
                    None => eprintln!("Process terminated by signal"),
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
