# firetest

`firetest` is a tool for _easily_ running binaries in a virtual machine. It uses [firecracker_spawn](https://github.com/DavidVentura/firecracker-spawn) to spawn virtual machines.

## Features

- Fast VM initialization using [Firecracker](https://github.com/firecracker-microvm/firecracker/tree/main)
- Static binary which embeds kernel (6.7), strace, and busybox for self-contained execution
- Can pass through arguments for tested binaries
- VSOCK communication for retrieving test results

## Prerequisites

Host:

- Being member of `kvm` (or being able to write to `/dev/kvm`) 
- Having the `vhost_vsock` module loaded
- If you want to use network, you need a `tap` interface to exist

Guest:
- No module loading, so all kernel requirements must be built-in
- There's no userland, so tested binaries must be static

## Usage

```
firetest <binary_path> [args...]
```

Where:
- `<binary_path>` is the path to the binary you want to run in the VM
- `[args...]` are optional arguments to pass to the binary

## Example

Here's an example of using `firetest` for running Cargo integration tests:

```bash
CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_RUNNER='firetest' time cargo test -- --ignored --nocapture direct
running 1 test
test trace_direct_connection ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s


real    0m0.607s
```

This command sets `firetest` as the runner for Cargo tests targeting the `x86_64-unknown-linux-musl` platform.

## How it works

1. `firetest` builds a custom initrd containing:
   - The user-provided binary
   - A custom init process
   - busybox
   - strace

2. It then spawns a Firecracker VM with:
   - 1 vCPU
   - 256 MiB of memory
   - The embedded Linux kernel
   - The custom initrd
   - VSOCK for communication

3. The custom init executes the user's binary, capturing the output.

4. Results are communicated back to the host via VSOCK.

## Building

To build the project, use make:

```bash
make
```

## Notes

- The project includes embedded binaries (kernel, busybox, strace) to minimize external dependencies.
- The VM is configured with minimal resources to ensure fast startup and execution.
- The firecracker people do *not* like executing `firecracker` without process isolation, seccomp, etc.

## License

MIT

## Acknowledgements

This project takes inspiration from [vmtest](https://github.com/danobi/vmtest)
