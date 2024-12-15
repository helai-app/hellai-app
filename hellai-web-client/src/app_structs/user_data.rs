use std::fmt;

use super::project_info::ProjectInfo;

#[allow(unused)]
pub struct UserData {
    pub id: i32,
    pub email: String,
    pub projects: Vec<ProjectInfo>,
}

impl fmt::Display for UserData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display user ID
        writeln!(f, "User ID: {}", self.id)?;

        // Display email, handling the Option
        writeln!(f, "Email: {}", &self.email);

        // Display each project
        writeln!(f, "Projects:")?;
        for project in &self.projects {
            writeln!(f, "  - {}", project)?;
        }

        Ok(())
    }
}
