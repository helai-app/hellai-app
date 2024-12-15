use std::fmt;

#[allow(unused)]
pub struct ProjectInfo {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub decoration_color: Option<String>,
}

impl fmt::Display for ProjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} (ID: {}), DESCRIPTION: {:?}, COLOR: {:?}",
            self.title, self.id, self.description, self.decoration_color
        )
    }
}
