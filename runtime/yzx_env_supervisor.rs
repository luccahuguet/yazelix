use std::{
    env,
    fs::File,
    process::{self, Command, Stdio},
    sync::atomic::{AtomicI32, Ordering},
};

static CHILD_PID: AtomicI32 = AtomicI32::new(0);

unsafe extern "C" {
    fn kill(pid: i32, signal: i32) -> i32;
    fn signal(signal: i32, handler: extern "C" fn(i32)) -> usize;
}

extern "C" fn forward_signal(signal_number: i32) {
    let child_pid = CHILD_PID.load(Ordering::Relaxed);
    if child_pid > 0 {
        unsafe {
            kill(child_pid, signal_number);
        }
    }
}

fn install_signal_forwarding() {
    for signal_number in [1, 2, 15] {
        unsafe {
            signal(signal_number, forward_signal);
        }
    }
}

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(program) = args.next() else {
        eprintln!("usage: yzx-env-supervisor <program> [args...]");
        process::exit(64);
    };
    let tty = File::open("/dev/tty").unwrap_or_else(|error| {
        eprintln!("yzx-env-supervisor: cannot open /dev/tty: {error}");
        process::exit(1);
    });
    install_signal_forwarding();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::from(tty))
        .spawn()
        .unwrap_or_else(|error| {
            eprintln!("yzx-env-supervisor: cannot start child: {error}");
            process::exit(127);
        });
    CHILD_PID.store(child.id() as i32, Ordering::Relaxed);
    let status = child.wait().unwrap_or_else(|error| {
        eprintln!("yzx-env-supervisor: cannot wait for child: {error}");
        process::exit(1);
    });
    CHILD_PID.store(0, Ordering::Relaxed);
    process::exit(status.code().unwrap_or(128));
}
