#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use ole_kit::cfb::header::Header;

    #[test]
    fn parse_doc() {
        let file = File::open("tests_rsc/hwp5.0.hwp").unwrap();
        let header = Header::new(&file);
        assert_eq!(header.signature().0, 0xe11ab1a1e011cfd0);
        assert_eq!(header.minor_version().0, 0x003E);
        assert_eq!(header.major_version().0, 0x0003);
        assert_eq!(header.byte_order(), 0xFFFE);
        assert_eq!(header.sector_shift().0, 0x0009);
        assert_eq!(header.mini_sector_shift().0, 0x0006);
        assert_eq!(header.no_of_directory_sectors(), None);
        assert_eq!(header.no_of_fat_sectors().0, 0x00000006);
        assert_eq!(header.first_directory_sector_location().0, 0x00000002);
        assert_eq!(header.transaction_signature_number(), 0x00000000);
        assert_eq!(header.mini_stream_cutoff_size(), 0x00001000);
        assert_eq!(header.first_mini_fat_sector_location().0, 0x00000007);
        assert_eq!(header.no_of_mini_fat_sectors().0, 0x00000004);
        assert_eq!(header.first_difat_sector_location().0, 0xFFFF_FFFE);
        assert_eq!(header.no_of_difat_sectors().0, 0x00000000);
    }
}