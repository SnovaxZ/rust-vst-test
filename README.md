# Myplug
A very basic granular(?) delay that works weird.

- Mode 1: delay

- Mode 2: delay that pitchshifts up then down

- Mode 3: wet signal only

- Mode 4: Messed up distortion sound sometimes works

- Mode 5: adds second delay that is controlled with the delay parameter to go slower than the real delay.

- Mode 6: switches around the n:th (set with mode6_ratio parameter) sample in the buffer



## Building

After installing [Rust](https://rustup.rs/), you can compile Myplug as follows:

RENAME THE CARGO FOLDER TO .cargo

```shell
cargo xtask bundle MYPLUG --release
```
