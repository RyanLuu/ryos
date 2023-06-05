# ryos

An operating system targeting RISC-V written in Rust, based on Stephen Marz's blog[^1] and MIT PDOS's xv6-riscv.[^2]

Because ryos is my attempt to learn more about Rust, RISC-V, and operating system concepts,
the source code generally aims for readability, rather than performance.

## Setup

### Ubuntu

1. Install Rust: https://www.rust-lang.org/tools/install

2. Enable unstable `#![features]` and add the RISC-V target:

   ```
   rustup default nightly
   rustup target add riscv64gc-unknown-none-elf
   cargo install cargo-binutils
   ```

3. Install QEMU:

   ```
   sudo apt install qemu-system-misc
   ```

4. Build and run:

   ```
   cargo run
   ```

[^1]: https://osblog.stephenmarz.com/index.html
[^2]: https://github.com/mit-pdos/xv6-riscv

