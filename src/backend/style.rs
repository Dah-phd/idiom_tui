use std::fmt::Debug;

#[allow(dead_code)]
pub trait StyleExt: Sized + PartialEq + Debug {
    type Color: Clone + Debug + PartialEq;
    type Attribute: Clone + Debug + PartialEq;
    fn update(&mut self, rhs: Self);
    fn set_attr(&mut self, attr: Self::Attribute);
    fn unset_attr(&mut self, attr: Self::Attribute);
    fn with_fg(self, color: Self::Color) -> Self;
    fn set_fg(&mut self, color: Option<Self::Color>);
    fn fg(color: Self::Color) -> Self;
    fn with_bg(self, color: Self::Color) -> Self;
    fn set_bg(&mut self, color: Option<Self::Color>);
    fn bg(color: Self::Color) -> Self;
    fn drop_bg(&mut self);
    fn add_slowblink(&mut self);
    fn slowblink() -> Self;
    fn add_bold(&mut self);
    fn bold() -> Self;
    fn add_ital(&mut self);
    fn ital() -> Self;
    fn add_reverse(&mut self);
    fn reversed() -> Self;
    fn reset_mods(&mut self);
    fn undercurle(&mut self, color: Option<Self::Color>);
    fn undercurled(color: Option<Self::Color>) -> Self;
    fn underline(&mut self, color: Option<Self::Color>);
    fn underlined(color: Option<Self::Color>) -> Self;
}
