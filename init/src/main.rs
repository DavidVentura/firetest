use shared::{send_message, Pid1Message};
use std::ffi::CString;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::{env, fs};
use vsock::{VsockStream, VMADDR_CID_HOST};

pub(crate) fn setup_environment() -> Result<(), io::Error> {
    fs::create_dir("/proc")?;
    mount_pseudo("/proc", "proc")?;
    fs::create_dir("/sys")?;
    mount_pseudo("/sys", "sysfs")?;
    mount_pseudo("/sys/kernel/tracing", "tracefs")?;
    // kernel may create /dev for us
    match fs::create_dir("/dev") {
        Ok(_) => mount_pseudo("/dev", "tmpfs")?,
        Err(_) => (),
    };
    mknod("/dev/null", 1, 3, libc::S_IFCHR)?;
    // mount_blockdevs()?;
    Ok(())
}

fn mount_pseudo(target: &str, fstype: &str) -> Result<(), io::Error> {
    mount(None, PathBuf::from(target), PathBuf::from(fstype))
}

pub(crate) fn mount(
    source: Option<PathBuf>,
    target: PathBuf,
    fstype: PathBuf,
) -> Result<(), io::Error> {
    let src = source
        .clone()
        .unwrap_or(PathBuf::from("none"))
        .into_os_string()
        .into_string()
        .unwrap();
    let tgt = target.into_os_string().into_string().unwrap();
    let fs = fstype.into_os_string().into_string().unwrap();
    let c_src = CString::new(src).unwrap();
    let c_tgt = CString::new(tgt).unwrap();
    let c_fstype = CString::new(fs).unwrap();

    let res = unsafe {
        libc::mount(
            c_src.as_ptr(),
            c_tgt.as_ptr(),
            c_fstype.as_ptr(),
            libc::MS_NOATIME | libc::MS_NODIRATIME,
            std::ptr::null(),
        )
    };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn mknod(path: &str, major: u32, minor: u32, mode: u32) -> Result<(), io::Error> {
    let devnum = libc::makedev(major, minor);
    let path = CString::new(path).unwrap();
    let res = unsafe { libc::mknod(path.as_ptr(), mode | 0o666, devnum) };
    if res == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

fn main() {
    let mut s = VsockStream::connect_with_cid_port(VMADDR_CID_HOST, 1234).unwrap();
    let args: Vec<String> = env::args().collect();
    send_message(
        &mut s,
        &Pid1Message::Booted {
            cmdline: "my cmdline".to_string(),
        },
    )
    .expect("unable to send booted message");
    println!("sent stuff");

    setup_environment().expect("was not able to set up /proc /sys /dev");

    // ugh - netlink?
    let _ = std::process::Command::new("/busybox")
        .args(&["ip", "link", "set", "lo", "up"])
        .spawn()
        .expect("Cannot bring interface 'lo' up");
    let _ = std::process::Command::new("/busybox")
        .args(&["ip", "route", "add", "default", "dev", "lo"])
        .spawn()
        .expect("Cannot add default route to 'lo'");

    let mut cmd = std::process::Command::new(&args[1]);
    if args.len() > 2 {
        cmd.args(&args[2..]);
    }

    std::thread::sleep(std::time::Duration::from_millis(10));
    match cmd.output() {
        Ok(r) => {
            println!("xgoood");
            send_message(
                &mut s,
                &Pid1Message::UserProcessFinished {
                    stdout: r.stdout,
                    stderr: r.stderr,
                    exit_code: r.status.code().unwrap(),
                },
            )
            .expect("unable to send msg");
        }
        Err(e) => {
            // TODO
            println!("xxxxxxxx {e}\n\n\n");
        }
    }
    std::thread::sleep(std::time::Duration::from_millis(10));
}
