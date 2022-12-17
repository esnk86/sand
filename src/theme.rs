pub struct Theme(pub u32, pub u32, pub u32);

pub enum ThemeId {
    Sandshell,
}

impl Theme {
    pub fn get(id: ThemeId) -> Theme {
        match id {
            ThemeId::Sandshell  => Theme(0x6b573d, 0x7a6e5e, 0xd8ccbb),
        }
    }
}
