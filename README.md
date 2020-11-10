# file-organizer

Simple rust application to organize files in a giving directory

## Requirement

Rust needs to be installed in order to use this utility

## Installation

```sh
cargo build --release

target/release/file_type 'path-to-organize'
```

## Motivation

My download's folder was cluttered and required organization of its files so 
I decided to write an simple command line application in rust to tidy up things

NB: Note that the follow directories need to be created in the desired directory
to function

- Multimedia    (For multimedia files: such as audio, videos etc)
- Docs          (For documents, pdf etc)
- Compressed    (For archives)
- Misc          (For others)