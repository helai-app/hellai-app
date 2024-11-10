use core_error::core_errors::CoreErrors;
use sea_orm::{DbBackend, DbConn, FromQueryResult, Statement};

/// User Company with projects
pub struct UserCompany {
    pub id: i32,
    pub name: String,
    pub name_alias: String,
    pub user_role: UserRole,
    pub description: Option<String>,
    pub contact_info: Option<String>,
    pub company_projects: Vec<UserCompanyProjects>,
}

pub struct UserCompanyProjects {
    pub id: i32,
    pub user_role: UserRole,
    pub company_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub decoration_color: Option<String>,
}

pub struct UserRole {
    pub id: i32,
    pub name: String,
    pub level: i32,
}

#[derive(Debug, FromQueryResult)]
struct UserCompanyQueryResult {
    // Fields from companies
    company_id: i32,
    company_name: String,
    name_alias: String,
    company_description: Option<String>,
    contact_info: Option<String>,

    // Fields from roles (user's role in the company)
    role_id: i32,
    role_name: String,
    role_level: i32,

    // Fields from projects (only present when company_id is specified)
    project_id: Option<i32>, // Optional because it might be NULL if no projects are joined
    project_title: String,
    project_description: Option<String>,
    project_decoration_color: Option<String>,
}

/// Provides methods for querying and manipulating projects and user roles.
pub struct CompaniesQuery;

impl CompaniesQuery {
    /// Retrieves a single company with its projects, along with the user's role in the company and each project.
    /// If `company_id` is `None`, retrieves the first company the user has access to, without project details.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `user_id` - The ID of the user.
    /// * `company_id` - An optional filter for a specific company ID. If provided, retrieves this company and its projects.
    ///
    /// # Returns
    ///
    /// An `Option<UserCompany>` structure containing information about the company, the user's role in that company,
    /// and a list of projects within the company where the user has access. If the user has access to the company,
    /// they automatically have access to all its projects. Returns `None` if no company is found for the user.
    ///
    /// Returns an error of type `CoreErrors` if the query fails.
    pub async fn get_company_with_projects(
        db: &DbConn,
        user_id: i32,
        company_id: Option<i32>,
    ) -> Result<Option<UserCompany>, CoreErrors> {
        // Define the SQL query based on whether `company_id` is provided.
        // If a specific company ID is provided, fetch both company and project details.
        let sql = if let Some(cid) = company_id {
            // SQL query for fetching company details and its associated projects.
            r#"
        SELECT
            c.id AS company_id,
            c.name AS company_name,
            c.name_alias,
            r.id AS role_id,
            r.name AS role_name,
            r.level AS role_level,
            c.description,
            c.contact_info,
            p.id AS project_id,
            p.title AS project_title,
            p.description AS project_description,
            p.decoration_color AS project_decoration_color
        FROM
            companies c
            INNER JOIN user_company uc ON uc.company_id = c.id
            INNER JOIN roles r ON r.id = uc.role_id
            LEFT JOIN projects p ON p.company_id = c.id
        WHERE
            uc.user_id = $1
            AND c.id = $2
        "#
        } else {
            // SQL query for fetching only the first company without project details.
            r#"
        SELECT
            c.id AS company_id,
            c.name AS company_name,
            c.name_alias,
            r.id AS role_id,
            r.name AS role_name,
            r.level AS role_level,
            c.description,
            c.contact_info
        FROM
            companies c
            INNER JOIN user_company uc ON uc.company_id = c.id
            INNER JOIN roles r ON r.id = uc.role_id
        WHERE
            uc.user_id = $1
        LIMIT 1
        "#
        };

        // Prepare the SQL statement with parameters. If `company_id` is provided, add it as a parameter.
        let stmt = if let Some(cid) = company_id {
            Statement::from_sql_and_values(DbBackend::Postgres, sql, [user_id.into(), cid.into()])
        } else {
            Statement::from_sql_and_values(DbBackend::Postgres, sql, [user_id.into()])
        };

        // Execute the query and retrieve the results as a vector of `UserCompanyQueryResult`.
        let query_results = UserCompanyQueryResult::find_by_statement(stmt)
            .all(db)
            .await?;

        // If no results are found, return `None`.
        if query_results.is_empty() {
            return Ok(None);
        }

        // Initialize a single `UserCompany` to collect data.
        let mut user_company = None;

        for record in query_results {
            // If `user_company` is not initialized yet, create it with company-level data.
            if user_company.is_none() {
                user_company = Some(UserCompany {
                    id: record.company_id,
                    name: record.company_name.clone(),
                    name_alias: record.name_alias.clone(),
                    description: record.company_description.clone(),
                    contact_info: record.contact_info.clone(),
                    user_role: UserRole {
                        id: record.role_id,
                        name: record.role_name.clone(),
                        level: record.role_level,
                    },
                    company_projects: Vec::new(),
                });
            }

            // If project details are available, add them to `company_projects`.
            if let Some(project_id) = record.project_id {
                if let Some(ref mut company) = user_company {
                    company.company_projects.push(UserCompanyProjects {
                        id: project_id,
                        company_id: record.company_id,
                        title: record.project_title.clone(),
                        description: record.project_description.clone(),
                        decoration_color: record.project_decoration_color.clone(),
                        user_role: UserRole {
                            id: record.role_id,
                            name: record.role_name.clone(),
                            level: record.role_level,
                        },
                    });
                }
            }
        }

        // Return the `UserCompany` wrapped in `Some`, or `None` if no data was found
        Ok(user_company)
    }
}
