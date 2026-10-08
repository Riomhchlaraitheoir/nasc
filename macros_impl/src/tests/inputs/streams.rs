
#[rpc]
trait StreamsRPC {
    fn receive() -> impl Stream<Item=Foo>;
    fn send(stream: impl Stream<Item=Foo>);

    fn multiple(option: Option<impl Stream<Item=Foo>>, tuple: (Foo, impl Stream<Item=Foo>)) -> Result<impl Stream<Item=Bar>, Error>;
}