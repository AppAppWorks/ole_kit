#[cfg(test)]
mod tests {
    use ole_kit::cfb::Cfb;

    #[test]
    fn read_stream_bytes() {
        let cfb = Cfb::from_path("tests_rsc/testing.doc").unwrap();
        let word_document_bytes = cfb.stream_bytes("WordDocument");
        assert!(word_document_bytes.is_some());
        assert_eq!(word_document_bytes.as_ref().map(Vec::len), Some(4096));
    }
}