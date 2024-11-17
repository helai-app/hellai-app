use core_database::{
    entity::user_company,
    queries::{
        companies_query::CompaniesQuery,
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
// pub async fn check_tasks_permission(
//     conn: &DbConn,
//     user_id: i32,
//     task_id: i32,
// ) -> Result<UserProject, Status> {
//     // Retrieve the user's role in the project from the database
//     let user_project = TasksQuery::get_user_task(conn, user_id, task_id).await?;

//     match user_project {
//         Some(project) => return Ok(project),
//         None => Err(Status::permission_denied("permission_denied")),
//     }
// }
