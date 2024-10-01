use cpio::{newc, NewcBuilder};
//use firecracker_spawn::{Disk, NetConfig, Vm};
use firecracker_spawn::Vm;
use std::env;
use std::error::Error;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::{io, thread};

mod utils;

const BUSYBOX_BYTES: &[u8] = include_bytes!("../bins/busybox.zst");
const STRACE_BYTES: &[u8] = include_bytes!("../bins/strace.zst");
const KERNEL_BYTES: &[u8] = include_bytes!("../bins/vmlinux.zst");
const INIT_BYTES: &[u8] =
    include_bytes!("../../init/target/x86_64-unknown-linux-musl/release/init.zst");

fn build_initrd(bin_path: &str, bin_name: &str) -> Result<File, Box<dyn Error>> {
    let mut cpio_file = tempfile::tempfile()?;
    let user_init_bytes = fs::read(bin_path)?;
    let cpio_user_program = NewcBuilder::new(bin_name)
        .mode(0o777)
        .set_mode_file_type(newc::ModeFileType::Regular);
    let mut fp = cpio_user_program.write(&mut cpio_file, user_init_bytes.len() as u32);
    fp.write_all(&user_init_bytes)?;
    fp.finish()?;

    for (fname, compressed) in &[
        ("init", INIT_BYTES),
        ("strace", STRACE_BYTES),
        ("busybox", BUSYBOX_BYTES),
    ] {
        let entry = NewcBuilder::new(fname)
            .mode(0o777)
            .set_mode_file_type(newc::ModeFileType::Regular);
        let init_dec = utils::zstd_dec(&compressed)?;
        let mut fp = entry.write(&mut cpio_file, init_dec.len() as u32);
        fp.write_all(&init_dec)?;
        fp.finish()?;
    }

    newc::trailer(&mut cpio_file)?;
    cpio_file.flush()?;
    Ok(cpio_file)
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let bin_path = &args[1];
    let bin_name = Path::new(&args[1]).file_name().unwrap().to_str().unwrap();

    let cpio_file = build_initrd(bin_path, bin_name).expect("Could not build initrd");
    let str_args = args[2..].join(" ");

    let kernel =
        utils::zstd_buf_to_fd("kernel", KERNEL_BYTES).expect("Cannot mount kernel as memfd");

    let vsock_path = "/tmp/test.v.sock";
    let port = 1234;
    let vsock_listener = format!("{}_{}", vsock_path, port);
    let _ = fs::remove_file(vsock_path);
    let _ = fs::remove_file(&vsock_listener);

    let kernel_cmdline = format!(
        "quiet panic=-1 reboot=t rdinit=/init -- /{bin_name} {str_args}" //"quiet panic=-1 reboot=t rdinit=/strace -- -f /init /{bin_name} {str_args}"
    );

    let v = Vm {
        vcpu_count: 1,
        mem_size_mib: 256,
        kernel,
        kernel_cmdline,
        rootfs: None,
        initrd: Some(cpio_file),
        extra_disks: vec![],
        net_config: None,
        use_hugepages: false,
        vsock: Some(vsock_path.to_string()),
    };
    let handle = thread::spawn(move || {
        let listener = UnixListener::bind(vsock_listener).unwrap();
        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buf = Vec::new();
                    // this read_to_end waits for the conn to close
                    stream.read_to_end(&mut buf).unwrap();
                    let s = String::from_utf8(buf).unwrap();
                    println!("got {s}");
                    break;
                }
                Err(_) => panic!("uh"),
            }
        }
    });
    // TODO this could go to a different log for kernel
    v.make(Box::new(io::sink())).unwrap();
    //v.make(Box::new(io::stdout())).unwrap();
    handle.join().unwrap();
}
