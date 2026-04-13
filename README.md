JUNKCC

A Compiler for a subset of C based on the excellent book "Writing a C Compiler" by Nora Sandler

Features:
  * Written in Rust
  * Generates code for Linux x86_64
  * Uses the following existing binaries from the Linux distro:
      * cpp -- the C preprocessor
      * as -- the GNU assembler from the binutils package
      * ld -- the GNU linker, also from the binutils package
  * Links against the installed GNU libc 

