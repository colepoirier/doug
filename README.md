# doug

Doug is a WIP semi-automated to full manual VLSI Analog and Mixed Signal CAD design tool using the rust game engine project bevy and https://github.com/dan-fritchman/Layout21.

To run, simply ensure you are running a recent nightly rustc, just run `rustup default nightly`.

Then `cargo r --release`.

You may run into issues due to errors from missing bevy's native dependencies for audio on your OS. See <https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md>.

For now, I will only be focusing on testing on linux, but bevy has good cross-platform support so you should not have any issues running on Windows 10 or MacOS.
