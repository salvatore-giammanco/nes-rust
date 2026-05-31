pub enum MaskFlags {
    Greyscale,
    ShowBackground,
    ShowSprites,
    BackgroundRendering,
    SpriteRendering,
    EmphasizeRed,
    EmphasizeGreen,
    EmphasizeBlue,
}

pub struct MaskRegister {
    pub status: u8,
}

pub struct FlagMask {
    set: u8,
    unset: u8,
}
