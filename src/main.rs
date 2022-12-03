#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unused)]
#[cfg(test)]
mod tests;
use foxmd::*;

// Handle command line args here in main; but make every routine that can be called by cla available in lib.rs
// TODO Update Table of Contents to handle recursive dirs
// TODO Update writing HTML header to change CSS file inclusion to handle recursive dirs
fn main() {
    parse_fmds_in_dir_recursively(".");
}
