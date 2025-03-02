#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Variable(u8);
impl Variable{
    pub fn new(name: u8)->Self{Self(name)}
}