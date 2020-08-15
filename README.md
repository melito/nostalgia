nostalgia
=========

This is a crate that provides syntactic sugar for interacting with certain database systems.

Currently it allows you to model and query data in an lmdb database using simple conventions.

This project is very much still a work in progress and currently only supports the Lightning Memory-Mapped Database (lmdb).

## Roadmap

### Features

  * More syntactic sugar

  * Pluggable backends.  Support for databases other than lmdb

  * Pluggable serialization models

  * Ability to force struct layout conformity for compatibility with databases created in other languages.

    For now marking a struct with `#[repr(C)]` should work

  * Ability to configure storage backends in a simple manner
  
  * Performance benchmark suite

  * Performance improvements based on feedback from continuous automated benchmarks
