pub struct FlagMask {
    pub set: u8,
    pub unset: u8,
}

pub trait BitFlags<T> {
    fn new() -> Self;
    fn get_mask(&self, flag: T) -> FlagMask;
    fn get_status(&self) -> u8;

    fn set_from_byte(&mut self, byte: u8);

    fn set_flag(&mut self, flag: T, bit: bool) {
        match bit {
            true => self.set_from_byte(self.get_status() | self.get_mask(flag).set),
            false => self.set_from_byte(self.get_status() & self.get_mask(flag).unset),
        }
    }

    fn get_flag(&self, flag: T) -> bool {
        let check = self.get_mask(flag).set & self.get_status();
        check.count_ones() != 0
    }
}
