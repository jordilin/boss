# Boss

Boss is a very simple abstraction over crossbeam-channel. It
implements a fan-out pattern to distribute execution of functions
across multiple cores. A function accepts any type and gets executed
whenever new data is being sent. It's up to the function to handle any
potential errors that might ocur.

See examples folder for a very simple example.

```bash
cargo run --example main
```

## License

This project is licensed under either of

* Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
* MIT license ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
