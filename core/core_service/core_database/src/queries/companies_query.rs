use core_error::core_errors::CoreErrors;
use rand::Rng;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DbBackend, DbConn,
    DeleteResult, EntityTrait, FromQueryResult, IntoActiveModel, QueryFilter, Set, Statement,
};

use crate::entity::{companies, sea_orm_active_enums::AccessLevelType, user_company};

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
        // Define the SQL query depending on whether a specific `company_id` is provided.
        let sql = if company_id.is_some() {
            // Fetch both company and project details when `company_id` is specified.
            r#"
            SELECT
                c.id AS company_id,
                c.name AS company_name,
                c.name_alias,
                r.id AS role_id,
                r.name AS role_name,
                r.level AS role_level,
                c.description AS company_description,
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
            // Fetch only the first company without project details if `company_id` is not provided.
            r#"
            SELECT
                c.id AS company_id,
                c.name AS company_name,
                c.name_alias,
                r.id AS role_id,
                r.name AS role_name,
                r.level AS role_level,
                c.description AS company_description,
                c.contact_info,
                p.id AS project_id,
                p.title AS project_title,
                p.description AS project_description,
                p.decoration_color AS project_decoration_color
            FROM
                companies c
                INNER JOIN user_company uc ON uc.company_id = c.id
                LEFT JOIN roles r ON r.id = uc.role_id
                LEFT JOIN projects p ON p.company_id = c.id
            WHERE
            uc.user_id = $1
            LIMIT 1;
            "#
        };

        // Prepare the SQL statement with parameters based on the presence of `company_id`.
        let stmt = match company_id {
            Some(cid) => Statement::from_sql_and_values(
                DbBackend::Postgres,
                sql,
                [user_id.into(), cid.into()],
            ),
            None => Statement::from_sql_and_values(DbBackend::Postgres, sql, [user_id.into()]),
        };
        println!("Step 1.");

        // Execute the query and retrieve results as `UserCompanyQueryResult`.
        let query_results = UserCompanyQueryResult::find_by_statement(stmt)
            .all(db)
            .await?;

        println!("Step 2.");

        // Return `None` if no results are found.
        if query_results.is_empty() {
            return Ok(None);
        }

        // Initialize `UserCompany` to aggregate data from the query results.
        let mut user_company = None;

        println!("Step 3.");

        // Process each record in the query result to build the `UserCompany` structure.
        for record in query_results {
            // Initialize `user_company` with company-level data if not yet done.
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

            println!("Step 4.");

            // Add project details if available to `company_projects`.
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

        // Return the `UserCompany` wrapped in `Some`, or `None` if no data was found.
        Ok(user_company)
    }

    /// Creates a new company and assigns the specified user to that company with a "limited" access level.
    ///
    /// This function uses Common Table Expressions (CTEs) to first insert a new company record into the `companies` table.
    /// It then assigns the user to the new company with a "limited" access level in the `UserCompany` table.
    /// The `name_alias` field is automatically generated from the `name` by converting it to a lowercase, alphanumeric identifier.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `user_id` - The ID of the user to be assigned to the new company.
    /// * `name` - The name of the new company.
    /// * `description` - A brief description of the company.
    /// * `contact_info` - Contact information for the company.
    ///
    /// # Returns
    ///
    /// A `Result<companies::Model, CoreErrors>` containing the newly created company model, including fields:
    /// `id`, `name`, `name_alias`, `description`, and `contact_info`. The company model is returned if creation is successful.
    ///
    /// Returns an error of type `CoreErrors` if the query fails.
    ///
    /// # Example
    ///
    /// ```
    /// let company = create_new_company(&db, user_id, "Tech Corp", "An innovative tech company.", "contact@techcorp.com").await?;
    /// println!("New company created with ID: {}", company.id);
    /// ```

    pub async fn create_new_company(
        db: &DbConn,
        user_id: i32,
        name: String,
        description: Option<String>,
        contact_info: Option<String>,
    ) -> Result<companies::Model, CoreErrors> {
        let mut attempts = 0;

        loop {
            // Generate the name alias, adding a random suffix if needed for retry attempts.
            let name_alias = if attempts == 0 {
                convert_to_identifier(name.as_str())
            } else {
                format!(
                    "{}{}",
                    convert_to_identifier(name.as_str()),
                    rand::thread_rng().gen_range(1..1000)
                )
            };

            println!("Test {}", name_alias);

            // Step 1: Check if `name_alias` already exists in the `companies` table.
            let check_alias_sql =
                "SELECT EXISTS (SELECT 1 FROM companies WHERE name_alias = $1) AS alias_exists;";

            let check_alias_stmt = Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                check_alias_sql,
                vec![name_alias.clone().into()], // $1 - Company alias to check
            );

            if let Some(alias_check_result) = db.query_one(check_alias_stmt).await? {
                let alias_exists: bool = alias_check_result.try_get("", "alias_exists")?;
                if alias_exists {
                    // If alias exists, retry with a new alias
                    attempts += 1;
                    continue;
                }
            }

            // Step 2: Insert the new company and assign the user to the company in `UserCompany` table.
            let insert_sql = r#"
            WITH new_company AS (
                INSERT INTO companies (name, name_alias, description, contact_info)
                VALUES ($1, $2, $3, $4)
                RETURNING id, name, name_alias, description, contact_info
            ),
            inserted_user_company AS (
                INSERT INTO user_company (user_id, company_id, role_id, access_level)
                SELECT $5, id, 1, 'full'  -- Assign "full" access with role_id 1 for the user in the new company
                FROM new_company
                RETURNING id
            )
            SELECT id, name, name_alias, description, contact_info
            FROM new_company;
        "#;

            let insert_stmt = Statement::from_sql_and_values(
                DatabaseBackend::Postgres,
                insert_sql,
                vec![
                    name.clone().into(),         // $1 - Company name
                    name_alias.clone().into(),   // $2 - Company alias
                    description.clone().into(),  // $3 - Company description
                    contact_info.clone().into(), // $4 - Company contact information
                    user_id.into(),              // $5 - User ID for UserCompany assignment
                ],
            );

            // Execute the insert statement
            match db.query_one(insert_stmt).await {
                Ok(Some(row)) => {
                    // Successfully created company, extract details from the result
                    let company = companies::Model {
                        id: row.try_get("", "id")?,
                        name: row.try_get("", "name")?,
                        name_alias: row.try_get("", "name_alias")?,
                        description: row.try_get("", "description")?,
                        contact_info: row.try_get("", "contact_info")?,
                    };
                    return Ok(company);
                }
                Ok(None) => {
                    // If no row was returned, return an error indicating the failure
                    return Err(CoreErrors::DatabaseServiceError(
                        "Failed to create company".to_string(),
                    ));
                }
                Err(err) => return Err(err.into()), // For other errors, convert them into CoreErrors
            }
        }
    }

    /// Retrieves a specific user's association with a company based on user and company IDs.
    ///
    /// This function checks if a given user is associated with a specific company by querying
    /// the `user_company` table. If the user-company relationship exists, it returns the `user_company`
    /// model; otherwise, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user whose company association is being queried.
    /// * `company_id` - The ID of the company to check for association with the user.
    ///
    /// # Returns
    ///
    /// * `Result<Option<user_company::Model>, CoreErrors>` - Returns an `Option` containing the
    ///   `user_company::Model` if the user-company association exists, or `None` if not found.
    ///   Returns `CoreErrors` in case of any database query failure.
    pub async fn get_user_company(
        db: &DbConn,
        user_id: i32,
        company_id: i32,
    ) -> Result<Option<user_company::Model>, CoreErrors> {
        let company: Option<user_company::Model> = user_company::Entity::find()
            .filter(user_company::Column::UserId.eq(user_id))
            .filter(user_company::Column::CompanyId.eq(company_id))
            .one(db)
            .await?;

        Ok(company)
    }

    /// Adds a user to a company or updates their role if they are not already associated with the company.
    ///
    /// This function checks if the specified user is already assigned a role within the given company.
    /// If they are, it does not modify the existing role. If they are not yet associated,
    /// it creates a new association with the "Manager" role (role ID 3) and limited access level.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection reference.
    /// * `user_id` - The ID of the user to be added or updated within the company.
    /// * `company_id` - The ID of the company to which the user is being added.
    ///
    /// # Returns
    ///
    /// * `Result<user_company::Model, CoreErrors>` - Returns the `user_company::Model` representing
    ///   the user's association with the company, whether it was created or found.
    pub async fn add_user_to_company(
        db: &DbConn,
        user_id: i32,
        company_id: i32,
    ) -> Result<user_company::Model, CoreErrors> {
        // Step 1: Check if the user already has an existing role in the specified company
        if let Some(existing_access) = user_company::Entity::find()
            .filter(user_company::Column::UserId.eq(user_id)) // Filter by user ID
            .filter(user_company::Column::CompanyId.eq(company_id)) // Filter by company ID
            .one(db)
            .await?
        {
            // User already associated with the company; return the existing record without changes
            return Ok(existing_access);
        }

        // Step 2: No existing role found; create a new association with "Manager" role and limited access
        let new_access = user_company::ActiveModel {
            user_id: Set(user_id),                       // Set the user ID
            company_id: Set(company_id),                 // Set the company ID
            role_id: Set(3),                             // Assign role ID 3 for "Manager"
            access_level: Set(AccessLevelType::Limited), // Assign limited access level
            ..Default::default()                         // Set other fields to default values
        };

        // Step 3: Insert the new role assignment into the database
        let result = new_access.insert(db).await?;

        // Step 4: Return the newly created record
        Ok(result)
    }

    /// Removes a user from a specified company if they have an existing role or association.
    ///
    /// This function checks if the user is associated with the given company.
    /// If an association is found, it deletes the user’s record from the company.
    /// If no association exists, it returns a `CoreErrors::DatabaseServiceError`.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user to be removed from the company.
    /// * `company_id` - The ID of the company from which the user is being removed.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if the user is successfully removed,
    ///   or a `CoreErrors::DatabaseServiceError` if no association exists or another error occurs.
    pub async fn remove_user_from_company(
        db: &DbConn,
        user_id: i32,
        company_id: i32,
        request_user_id_lvl: Option<i32>,
    ) -> Result<(), CoreErrors> {
        // Step 1: Check if the user has an existing role in the specified company
        let existing_access = match request_user_id_lvl {
            Some(level) => {
                // If `request_user_id_lvl` is `Some`, include the role level filter
                user_company::Entity::find()
                    .filter(user_company::Column::UserId.eq(user_id)) // Filter by user ID
                    .filter(user_company::Column::CompanyId.eq(company_id)) // Filter by company ID
                    .filter(user_company::Column::RoleId.lte(level)) // Ensure role_id <= level
                    .one(db)
                    .await?
            }
            None => {
                // If `request_user_id_lvl` is `None`, exclude the role level filter
                user_company::Entity::find()
                    .filter(user_company::Column::UserId.eq(user_id)) // Filter by user ID
                    .filter(user_company::Column::CompanyId.eq(company_id)) // Filter by company ID
                    .one(db)
                    .await?
            }
        };

        // Step 2: If an association exists, delete it; otherwise, return an error
        match existing_access {
            Some(access) => {
                // Convert the record to an active model to enable deletion
                let active_role = access.into_active_model();

                // Delete the existing record from the database
                active_role.delete(db).await?
            }
            None => {
                // Return an error if no user-company association exists
                return Err(CoreErrors::DatabaseServiceError(
                    "User not associated with the specified company".to_string(),
                ));
            }
        };

        // Return success if the user was successfully removed
        Ok(())
    }

    pub async fn delete_company(db: &DbConn, company_id: i32) -> Result<(), CoreErrors> {
        // Step 1: Check if the user has an existing role in the specified company
        let company = companies::Entity::find_by_id(company_id).one(db).await?;

        match company {
            Some(company) => {
                let active_company = company.into_active_model();

                active_company.delete(db).await?
            }
            None => {
                // Return an error if no user-company association exists
                return Err(CoreErrors::DatabaseServiceError(
                    "User not associated with the specified company".to_string(),
                ));
            }
        };

        // Return success if the user was successfully removed
        Ok(())
    }

    /// Deletes all user associations from the specified company.
    ///
    /// This function removes all records from the `user_company` table where the `company_id` matches
    /// the specified company ID, effectively removing all users from the company.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `company_id` - The ID of the company from which all users will be removed.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if all users were successfully removed,
    ///   or a `CoreErrors` error if the operation fails.
    pub async fn delete_all_users_from_company(
        db: &DbConn,
        company_id: i32,
    ) -> Result<(), CoreErrors> {
        // Delete all user-company associations for the specified company ID
        let _: DeleteResult = user_company::Entity::delete_many()
            .filter(user_company::Column::CompanyId.eq(company_id)) // Filter by company ID
            .exec(db)
            .await?;
        Ok(())
    }
}

fn convert_to_identifier(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric()) // Keep only alphanumeric characters
        .collect::<String>()
        .to_lowercase() // Convert to lowercase
}
