# rpc

This macro should be applied to trait definition to make it the base of an RPC service

It consumes the macro and provides a number of generated types

For example:

```rust
#[rpc]
trait Example {
    fn call(arg:u8) -> String;
}
```

This is a simplified, conceptual idea of the generated code
```rust
struct Example;

impl Rpc for Example {
    
}

impl Example {
    fn client(client: Client, format: impl Format) -> impl ExampleClient {
        
    }
}

/// Anything implementing this trait also implements `impl IntoHandler<Example>`
trait ExampleServer {
    type PanicStrategy;
    async fn call(&self, arg: u8) -> String;
}

trait ExampleClient {
    /// This function's future is a special future, that allows setting timeout,
    /// see generated docs for more detail
    async fn call(&self, arg: u8) -> Result<String, ClientError>;
}
```

For details of how to interact with the generated items see [Crate docs](crate)

## Arguments

This macro takes some arguments alongside the trait definition:

- `trait_rpc`: A path to the actual `trait_rpc` crate (default: `::trait_rpc`)
- `panic_handler`: The strategy for handling panics on this service, see [`panic_handler`](crate::panic_handler)

eg:
```rust
#[rpc(trait_rpc = custom::path::to::trait_rpc, panic_handler = catch)]
trait Service {
    fn call();
}
```

## Generics

Generics are supported on the trait level, method generics are not supported

## Parameters and Return Types

### Simple types
Any concrete type can be used in argument/return type position

However, they must be serializable, this means that you will only be able to use this service with formats that 
support the parameter and return types

### Streams

A special case is made for `impl Stream<Item=<type>>`

This can be used both in the parameters and in the return type, it can be nested within `Option`, `Result` or 
tuples to any depth

eg: 
```rust,ignore
fn call() -> Result<Option<(String, impl Stream<Item=u32>)>>;
```

For details on how streams function see [Crate docs](crate#Streams)

### Services

Additional services can be spawned using `impl Rpc<SeviceName>`, this too can be used in either the parameters or the
return type

For details on how services function see [Crate docs](crate#Services)
