# Boss

Boss is a very simple abstraction over crossbeam-channel. It
implements a fan-out pattern to distribute execution of functions
across multiple cores. A function accepts any type and gets executed
whenever new data is being sent. It's up to the function to handle any
potential errors that might ocur.

See examples folder for a very simple example.

cargo run --example main
