# Basic shell
This is a basic implementation of shell. It's including some main built-in commands, like `cp` or `cat`. 
The shell will work only in Windows, as some functions are based on Windows API and settings. 

The purpose of building such tool is:
- Practice. After reading Rust book, it's a good idea to try to build something by your hands.
- Get some grasp of buidling command line applications.
- Have fun 
- 
## Building

Clone this repository and then run
```
cd shell
cargo build
cargo run
```

If you want more optimized binary file run:
```
cargo run --release
```
