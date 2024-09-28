use core_error::core_errors::CoreErrors;
use sea_orm::{DbBackend, DbConn, FromQueryResult, Statement};

pub struct UserCompany {
    pub id: i32,
    pub name: String,
    pub user_role: UserCompanyRole,
}

pub struct UserCompanyRole {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, FromQueryResult)]
struct UserCompanyQueryResult {
    // Fields from UserCompanies
    user_company_id: i32,
    company_name: String,

    // Fields from CompanyRoles (UserCompanyRoles joined with CompanyRoles)
    role_id: i32,
    role_name: String,
    role_description: Option<String>,
}

pub struct CompanyQuery {}

impl CompanyQuery {
    pub async fn get_user_companies_with_roles(
        db: &DbConn,
        user_id: i32,
    ) -> Result<Vec<UserCompany>, CoreErrors> {
        let sql = r#"
        SELECT
            -- UserCompanies fields
            uc.id AS user_company_id,
            c.name AS company_name,

            -- CompanyRoles fields
            cr.id AS role_id,
            cr.name AS role_name,
            cr.description AS role_description

        FROM
            user_companies uc
            INNER JOIN companies c ON uc.company_id = c.id
            INNER JOIN user_company_roles ucr ON uc.user_id = ucr.user_id AND uc.company_id = ucr.company_id
            INNER JOIN company_roles cr ON ucr.company_role_id = cr.id
        WHERE
            uc.user_id = $1
    "#;

        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, sql, [user_id.into()]);

        let query_results = UserCompanyQueryResult::find_by_statement(stmt)
            .all(db)
            .await?;

        // Map the query results into a Vec<UserCompany>
        let mut user_companies: Vec<UserCompany> = vec![];

        for record in query_results {
            let user_company = UserCompany {
                id: record.user_company_id,
                name: record.company_name,
                user_role: UserCompanyRole {
                    id: record.role_id,
                    name: record.role_name,
                    description: record.role_description,
                },
            };

            user_companies.push(user_company);
        }

        Ok(user_companies)
    }
}
