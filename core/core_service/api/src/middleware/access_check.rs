use core_database::queries::companies_query::{CompaniesQuery, UserCompany};
use sea_orm::DbConn;
use tonic::Status;

// If user have access to company return it
// pub async fn check_company_permission(
//     conn: &DbConn,
//     user_id: i32,
//     company_id: i32,
// ) -> Result<UserCompany, Status> {
//     // Retrieve the user's role in the project from the database
//     let user_company = CompaniesQuery::get_user_company(conn, user_id, company_id).await?;

//     // Ensure that the user has "Owner" level permission (project_role_id == 1)
//     match user_company {
//         Some(company) => return company,
//         None => Err(Status::permission_denied("insufficient_rights")),
//     }
// }
