# Nasc

**Nasc** (Irish for *link* or *bond*) is a bidirectional RPC framework for Rust.
It allows you to define RPC services based on a trait definition.

## Core Features

- Rust trait definition: Define you service in native rust as a trait
- Lazy Streaming: service methods can receive/return streams, streams are evaluated lazily as they would be in normal rust
- sub-services: You root service definition can return other services, these services operate on the same connection as the root service 
- Bidirectional: Service methods are also able to receive service (client acting as server) making the connection bidirectional
- Transport agnostic: Several common transports supported out-of-the-box, with a `Connection` trait for you to implement your own support for any other transport

## Project Status
**Nasc is primarily a hobby project.**
While it is designed to be fully functional, robust and useful it was written first and foremost because it piqued my interest and I have several other projects that cna benefit from using this library

While people are welcome to use this if it fits their use case and any bug reports and feature requests are appreciated,
development follows my personal availability and interest.

I'm mostly making this available as a way to give back to open-source after benefiting from it for many years

## Project Goals

### No boilerplate
You should not need any boilerplate code when using this crate, once the connection is set up, you should be able to
just start using it without worrying about the connection

### Seamless code
This is intended to allow two different programs, wherever they may be running, to communicate seamlessly.
The code should, as much as possible, feel like the network isn't even there.

### Rust intrinsics respected
Following from [Seamless code](#seamless-code) above, all normal expected rust behaviours around futures, streams and RAII should work as expected as much as is possible

Some examples of what is meant by that:
- When streams are returned to the client, the server-side stream will not run until the client side stream is polled, then it will become inactive again once a value is outputted
- When a client-side future, stream or service client is dropped the server is notified so that it can cancel related server-side tasks and resources 

### Flexibility
This framework allows you to use custom transports and formats (for arguments and return values)

### Browser WASM support
Browser WebAssembly should be fully supported, at least for the client side.
Currently, this means that browser websockets can be used to start a connection 

### Stateful connection
The server side has convenient session-scoped state, simple keep state in the same type that implements the server side trait

## Support for languages other than rust
A common requirement of RPC frameworks is support for cross-language communication
**Nasc is intentionally designed to be an exclusive, Rust-only framework, and there are no plans to support other languages.**

This framework was designed from the ground up to exploit Rust's type system and async patterns.

Specifically here are some examples of Rust features used that do not cleanly translate to other languages
- RAII and resource lifetimes: Nasc maps client-side `drop()` to server-side clean-up freeing up resources when it is known that they will no longer be needed
- Trait definition, defining the entire service using a trait is great for rust since that definition maps smoothly into implementations on server and client side, this begins to break down if other languages are supported
- Async Streams: Nasc take heavy advantage of the lazy nature of rust's streams, not doing any work unless the client-side of the stream is ready to receive data

If you need/want a multi-language RPC framework, then there are many other great RPC frameworks that are much more flexible and will serve your needs much better (e.g.: gRPC).
Nasc chooses to trade multi-language support for a much deeper Rust support

## Installation
Add Nasc to your Cargo.toml
```toml
[dependencies]
nasc = { git = "https://github.com/Riomhchlaraitheoir/nasc" }
```
## License

Licensed under either of:

* Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://apache.org>)
* MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual-licensed as above, without any additional terms or conditions.