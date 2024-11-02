use std::fmt;

#[allow(unused)]
pub struct ProjectInfo {
    pub id: i32,
    pub name: String,
    pub role: UserProjectRole,
}

impl fmt::Display for ProjectInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (ID: {}), Role: {}", self.name, self.id, self.role)
    }
}

#[allow(unused)]
pub struct UserProjectRole {
    pub id: i32,
    pub name: String,
    pub description: String,
}

impl fmt::Display for UserProjectRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (ID: {}): {}", self.name, self.id, self.description)
    }
}
