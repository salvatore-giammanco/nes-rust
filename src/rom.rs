const NES_TAG: [u8; 4] = [0x4E, 0x45, 0x53, 0x1A];
const PRG_ROM_PAGE_SIZE: usize = 16384;
const CHR_ROM_PAGE_SIZE: usize = 8192;
const TRAINER_SIZE: usize = 512;

#[derive(Debug, PartialEq)]
pub enum Mirroring {
   Vertical,
   Horizontal,
   FourScreen,
}

#[derive(Debug, PartialEq)]
pub struct ROM {
    trainer: bool,
    mapper: u8,
    screen_mirroring: Mirroring,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}


impl ROM {
     pub fn from_file(file_path: &str) -> Result<Self, String> {
        let raw = std::fs::read(file_path).map_err(|e| e.to_string())?;
        Self::new(raw)
    }

    pub fn empty() -> Self {
        Self {
            trainer: false,
            mapper: 0,
            screen_mirroring: Mirroring::Horizontal,
            prg_rom: vec![0; 0x7FFF],
            chr_rom: vec![],
        }
    }

    pub fn new(raw: Vec<u8>) -> Result<Self, String> {
        // iNES Format
        if raw[0..4] != NES_TAG {
            return Err("Invalid NES file".to_string())
        }

        // iNES Version
        let version = raw[7] & 0b0000_1100 >> 2;
        if version != 0 {
            return Err("Only iNES version 1 supported".to_string())
        }

        // Mapper
        let mapper = raw[7] & 0b1111_0000 | raw[6] >> 4;
        if mapper != 0 {
            return Err("Rom's mapper not supported yet".to_string())
        }
        
        // Screen Mirroring
        let four_screen = (raw[6] & 0b0000_1000) >> 3;
        let mirroring = raw[6] & 0b0000_0001;
        let screen_mirroring = match (four_screen, mirroring) {
            (1, _) => Mirroring::FourScreen,
            (0, 0) => Mirroring::Horizontal,
            (0, 1) => Mirroring::Vertical,
            _ => unreachable!()
        };

        // Trainer
        let trainer: usize = ((raw[6] & 0b0000_0100) >> 2) as usize * TRAINER_SIZE;
        
        // PRG ROM
        let prg_rom_size: usize = raw[4] as usize * PRG_ROM_PAGE_SIZE;
        let prg_rom_start = 16 + trainer;
        let prg_rom = raw[prg_rom_start..prg_rom_start + prg_rom_size].to_vec();
        // CHR ROM
        let chr_rom_size: usize = raw[5] as usize * CHR_ROM_PAGE_SIZE;
        let chr_rom_start = prg_rom_start + prg_rom_size;
        let chr_rom = raw[chr_rom_start..chr_rom_start + chr_rom_size].to_vec();
        
        Ok(Self {
            trainer: trainer > 0,
            mapper,
            screen_mirroring,
            prg_rom,
            chr_rom,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rom_with_wrong_tag() {
        let rom = ROM::new(vec![0x00, 0x01, 0x02, 0x03]);
        assert!(rom.is_err());
        let e = rom.unwrap_err();
        assert_eq!(e, "Invalid NES file");
    }

    #[test]
    fn test_rom_with_wrong_version() {
        let rom = ROM::new(vec![0x4E, 0x45, 0x53, 0x1A, 0x00, 0x00, 0x00, 0x01]);
        assert!(rom.is_err());
        let e = rom.unwrap_err();
        assert_eq!(e, "Only iNES version 1 supported");
    }

    #[test]
    fn test_rom_with_unsupported_mapper() {
        let rom = ROM::new(vec![0x4E, 0x45, 0x53, 0x1A, 0x00, 0x00, 0x00, 0xF0]);
        assert!(rom.is_err());
        let e = rom.unwrap_err();
        assert_eq!(e, "Rom's mapper not supported yet");
    }

    #[test]
    fn test_rom_with_four_screen_mirroring() {
        let mut rom_raw: Vec<u8> = vec![0x00; 1024];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        rom_raw[6] = 0b0000_1001;
        let rom = ROM::new(rom_raw);
        assert_eq!(rom.unwrap().screen_mirroring, Mirroring::FourScreen);
    }

    #[test]
    fn test_rom_with_horizontal_mirroring() {
        let mut rom_raw: Vec<u8> = vec![0x00; 1024];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        let rom = ROM::new(rom_raw);
        assert_eq!(rom.unwrap().screen_mirroring, Mirroring::Horizontal);
    }

    #[test]
    fn test_rom_with_vertical_mirroring() {
        let mut rom_raw: Vec<u8> = vec![0x00; 1024];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        rom_raw[6] = 0b0000_0001;
        let rom = ROM::new(rom_raw);
        assert_eq!(rom.unwrap().screen_mirroring, Mirroring::Vertical);
    }

    #[test]
    fn test_rom_with_trainer() {
        let mut rom_raw: Vec<u8> = vec![0x00; 1024];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        rom_raw[6] = 0b0000_0100;
        let rom = ROM::new(rom_raw);
        assert!(rom.unwrap().trainer);
    }

    #[test]
    fn test_rom_without_trainer() {
        let mut rom_raw: Vec<u8> = vec![0x00; 1024];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        let rom = ROM::new(rom_raw);
        assert!(!rom.unwrap().trainer);
    }

    #[test]
    fn test_rom_with_prg_rom() {
        let mut rom_raw: Vec<u8> = vec![0x00; 16 + PRG_ROM_PAGE_SIZE];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        rom_raw[4] = 0x01;
        rom_raw[16..16 + PRG_ROM_PAGE_SIZE].copy_from_slice(&[0x01; PRG_ROM_PAGE_SIZE]);
        let rom = ROM::new(rom_raw);
        assert_eq!(rom.unwrap().prg_rom, vec![0x01; PRG_ROM_PAGE_SIZE]);
    }

    #[test]
    fn test_rom_with_chr_rom() {
        let mut rom_raw: Vec<u8> = vec![0x00; 16 + PRG_ROM_PAGE_SIZE + CHR_ROM_PAGE_SIZE];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        rom_raw[4] = 0x01;
        rom_raw[5] = 0x01;
        rom_raw[16 + PRG_ROM_PAGE_SIZE..16 + PRG_ROM_PAGE_SIZE + CHR_ROM_PAGE_SIZE].copy_from_slice(&[0x01; CHR_ROM_PAGE_SIZE]);
        let rom = ROM::new(rom_raw);
        assert_eq!(rom.unwrap().chr_rom, vec![0x01; CHR_ROM_PAGE_SIZE]);
    }

    #[test]
    fn test_rom_with_prg_rom_and_chr_rom_and_trainer() {
        let mut rom_raw: Vec<u8> = vec![0x00; 16 + TRAINER_SIZE + PRG_ROM_PAGE_SIZE + CHR_ROM_PAGE_SIZE];
        rom_raw[0..4].copy_from_slice(&NES_TAG);
        rom_raw[4] = 0x01;
        rom_raw[5] = 0x01;
        rom_raw[6] = 0b0000_0100;
        rom_raw[16 + TRAINER_SIZE..16 + TRAINER_SIZE + PRG_ROM_PAGE_SIZE].copy_from_slice(&[0x01; PRG_ROM_PAGE_SIZE]);
        rom_raw[16 + TRAINER_SIZE + PRG_ROM_PAGE_SIZE..16 + TRAINER_SIZE + PRG_ROM_PAGE_SIZE + CHR_ROM_PAGE_SIZE].copy_from_slice(&[0x02; CHR_ROM_PAGE_SIZE]);
        let rom = ROM::new(rom_raw).unwrap();
        assert_eq!(rom.prg_rom, vec![0x01; PRG_ROM_PAGE_SIZE]);
        assert_eq!(rom.chr_rom, vec![0x02; CHR_ROM_PAGE_SIZE]);
    }

}