use core_database::{
    entity::{notes, tasks, user_company},
    queries::{
        companies_query::CompaniesQuery,
        notes_query::NotesQuery,
        projects_query::{ProjectQuery, UserProject},
        tasks_query::TasksQuery,
    },
};
use sea_orm::DbConn;
use tonic::Status;

// If user have access to company return it
pub async fn check_company_permission(
    conn: &DbConn,
    user_id: i32,
    company_id: i32,
) -> Result<user_company::Model, Status> {
    // Retrieve the user's role in the project from the database
    let user_company = CompaniesQuery::get_user_company(conn, user_id, company_id).await?;

    // Ensure that the user has "Owner" level permission (project_role_id == 1)
    match user_company {
        Some(company) => return Ok(company),
        None => Err(Status::permission_denied("permission_denied")),
    }
}

// If user have access to project return it
pub async fn check_project_permission(
    conn: &DbConn,
    user_id: i32,
    project_id: i32,
) -> Result<UserProject, Status> {
    // Retrieve the user's role in the project from the database
    let user_project = ProjectQuery::get_user_project(conn, user_id, project_id).await?;

    match user_project {
        Some(project) => return Ok(project),
        None => Err(Status::permission_denied("permission_denied")),
    }
}

// If user have access to project return it
pub async fn check_tasks_permission(
    conn: &DbConn,
    user_id: i32,
    task_id: i32,
) -> Result<(tasks::Model, i32), Status> {
    // Retrieve the user's role in the project from the database
    let user_task = TasksQuery::get_user_task_with_access_lvl(conn, user_id, task_id).await?;

    match user_task {
        Some(task) => return Ok(task),
        None => Err(Status::permission_denied("permission_denied")),
    }
}

// First, verify that the note exists. Then, check access permissions based on the presence of a company ID:
// - If the project or task is associated with a company, validate access through the company's permissions.
// - Otherwise, ensure the user ID of the note matches the current user, allowing them to delete their own note.
pub async fn check_note_permission(
    conn: &DbConn,
    user_id: i32,
    note_id: i32,
) -> Result<notes::Model, Status> {
    // Retrieve the user's role in the project from the database
    let user_note = NotesQuery::check_user_permission(conn, note_id, user_id).await?;

    match user_note {
        Some(user_note) => return Ok(user_note),
        None => Err(Status::permission_denied("permission_denied")),
    }
}
