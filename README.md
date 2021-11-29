# organizer

Simple rust application to organize files in a given directory

## Requirement

Rust needs to be installed in order to use this utility

## Compilation

```bash
cargo build --release
```

## Execution
```bash
$ ./target/release/organizer --help
organizer 0.1.0
This application organizes the folder into categories (eg: Docs, Multimedia etc)

USAGE:
    organizer <path>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

ARGS:
    <path>    Path to organize
```

```bash
$ ./target/release/organizer ~/Downloads/
----[ Organizing (/Users/m3d/Downloads/) in Rust ]----
file: [/Users/m3d/Downloads/pwn.zip] type: [application/zip] [StatusCode: true]
file: [/Users/m3d/Downloads/Open_Sans.zip] type: [application/zip] [StatusCode: true]
```

## Motivation

My download's folder was cluttered and required organization of its files so 
I decided to write an simple command line application in rust to tidy up things

NB: Note that the follow directories will be created if not existing in the desired directory

* Multimedia    (For multimedia files: such as audio, videos etc)
* Docs          (For documents, pdf etc)
* Compressed    (For archives)
* Misc          (For others)
