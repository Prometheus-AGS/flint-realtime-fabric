# Tasks — p18-c007

- [x] Replace room_event_stream stream::empty() with a reqwest /sync long-poll loop (since-token, timeout, backoff)
- [x] Project Matrix room events → FederatedEvent; yield into the inbound stream
- [x] Handle auth (bearer), reconnect/backoff, and since-token persistence across polls
- [x] Update the stub comment; test the projection with a mocked /sync response
- [x] clippy/unwrap clean
