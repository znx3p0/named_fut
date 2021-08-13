# named fut

Creates a named future which can be used for async traits.
An example of this crate in use is [`qcomms`](https://github.com/znx3p0/qcomms), as it was the inspiration for this library

```rust
use async_std::io::prelude::Write;

#[named_fut(SendFut)]
pub async fn send_testing<'a, T>(st: &'a mut T) -> () where T: Write + Unpin {
    st.write(b"testing").await;
}

trait SendTesting {
    type Stream: Write + Unpin;
    fn get_stream(&mut self) -> Self::Stream;
    fn send_testing<'a>(&'a mut self) -> SendFut<'a, Self::Stream> {
        SendFut { st: self }
    }
}

```

