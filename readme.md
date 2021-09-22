ole_kit is a zero-copy Rust library for parsing [Microsoft OLE Compound File](https://docs.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/53989ce4-7b05-4f8d-829b-d08d6148375b).
It is a port from the Swift implementation used in [Aloud! - Text to Speech](https://apps.apple.com/us/app/aloud-text-to-speech-reader/id852033350).

## Usage

```rust
// create the CFB structure from a file
let cfb = Cfb::from_path("example/testing.doc");
// get the root storage directory entry
let root_entry = cfb.directories().next().map(|dir| dir.entry(0));
// read the bytes of the stream object named "WordDocument"
let word_document_bytes = cfb.stream_bytes("WordDocument");

```
