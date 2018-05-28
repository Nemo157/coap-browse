# `coap-browse`

A UI for making requests to [CoAP][] servers. Intended to be a
non-browser-extension based alternative to [Copper][]. Still very much a
prototype.

## Running

This is a sort of hybrid local/web application, similar to Electron based apps,
but instead of bundling a whole web browser it runs a small rendering engine in
your normal browser and streams virtual DOM changes to this over a WebSocket
from a Rust backend.

Start the backend server:

```sh
$ cargo run
```

Start the frontend server:

```sh
$ npm install
$ npm start
```

Load the frontend webpage in your browser

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.


[CoAp]: https://coap.technology
[Copper]: https://github.com/mkovatsc/Copper
