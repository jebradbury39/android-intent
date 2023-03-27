use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Flags: u32 {
        const GRANT_READ_URI_PERMISSION = 0b00000001;
    }
}
