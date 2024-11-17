use core_error::core_errors::CoreErrors;

use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DbBackend, DbConn,
    DeleteResult, EntityTrait, FromQueryResult, IntoActiveModel, QueryFilter, Set, Statement,
};

use crate::entity::sea_orm_active_enums::AccessLevelType;
use crate::entity::{projects, user_access};

/// Represents a project associated with a user, along with the user's role in that project.
pub struct UserProject {
    pub id: i32,
    pub company_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub decoration_color: Option<String>,
    pub user_role: UserProjectRole,
}

/// Represents the role a user has in a project.
pub struct UserProjectRole {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, FromQueryResult)]
struct UserProjectQueryResult {
    // Fields from user_projects
    project_id: i32,
    project_company_id: i32,
    project_title: String,
    project_description: Option<String>,
    project_decoration_color: Option<String>,

    // Fields from project_roles
    role_id: i32,
    role_name: String,
}

/// Provides methods for querying and manipulating projects and user roles.
pub struct ProjectQuery;

impl ProjectQuery {
    /// Retrieves all projects associated with a user, along with the user's role in each project.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `user_id` - The ID of the user.
    ///
    /// # Returns
    ///
    /// A `Vec` of `UserProject`, or an error of type `CoreErrors`.
    pub async fn get_user_projects_with_roles(
        db: &DbConn,
        user_id: i32,
    ) -> Result<Vec<UserProject>, CoreErrors> {
        let sql = r#"
            SELECT
                up.id AS user_project_id,
                p.name AS project_name,
                pr.id AS role_id,
                pr.name AS role_name,
            FROM
                user_projects up
                INNER JOIN projects p ON up.project_id = p.id
                INNER JOIN user_project_roles upr ON up.user_id = upr.user_id AND up.project_id = upr.project_id
                INNER JOIN project_roles pr ON upr.project_role_id = pr.id
            WHERE
                up.user_id = $1
        "#;

        let stmt = Statement::from_sql_and_values(DbBackend::Postgres, sql, [user_id.into()]);

        let query_results = UserProjectQueryResult::find_by_statement(stmt)
            .all(db)
            .await?;

        // Map the query results into a Vec<UserProject>
        let user_projects = query_results
            .into_iter()
            .map(|record| UserProject {
                id: record.project_id,
                company_id: record.project_company_id,
                title: record.project_title,
                description: record.project_description,
                decoration_color: record.project_decoration_color,
                user_role: UserProjectRole {
                    id: record.role_id,
                    name: record.role_name,
                },
            })
            .collect();

        Ok(user_projects)
    }

    /// Creates a new project within a specified company and optionally assigns user access.
    ///
    /// This function uses Common Table Expressions (CTEs) to insert a new project into the `projects` table.
    /// If the user's role ID is 3 or higher, the user is also added to the `user_access` table with full access.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `company_id` - The ID of the company to which the project belongs.
    /// * `title` - The title of the project.
    /// * `description` - A description of the project.
    /// * `decoration_color` - A decoration color for the project.
    /// * `user_id` - The ID of the user creating the project.
    /// * `role_id` - The role ID of the user. If the role ID is 3 or higher, user access is added.
    ///
    /// # Returns
    ///
    /// * `Result<projects::Model, CoreErrors>` - Returns the created project model on success,
    ///   or an error of type `CoreErrors` if the operation fails.
    pub async fn create_project(
        db: &DbConn,
        company_id: i32,
        title: String,
        description: String,
        decoration_color: String,
        user_id: i32,
        role_id: i32, // If role_id >= 3, add an entry in UserAccess
    ) -> Result<projects::Model, CoreErrors> {
        // SQL statement using Common Table Expressions (CTEs)
        let sql = r#"
        WITH new_project AS (
            INSERT INTO projects (company_id, title, description, decoration_color, created_at, updated_at)
            VALUES ($1, $2, $3, $4, CURRENT_TIMESTAMP AT TIME ZONE 'UTC', CURRENT_TIMESTAMP AT TIME ZONE 'UTC')
            RETURNING id, company_id, title, description, decoration_color, created_at, updated_at
        ),
        user_access AS (
            INSERT INTO user_access (user_id, company_id, project_id, role_id, access_level, created_at)
            SELECT $5, NULL, id, 1, 'full', CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
            FROM new_project
            WHERE $6 >= 3 -- Add to UserAccess only if role_id is 3 or higher
        )
        SELECT id, company_id, title, description, decoration_color, created_at, updated_at
        FROM new_project;
    "#;

        // Prepare the SQL statement with parameters
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            vec![
                company_id.into(),       // $1 - Company ID
                title.into(),            // $2 - Project title
                description.into(),      // $3 - Project description
                decoration_color.into(), // $4 - Decoration color
                user_id.into(),          // $5 - User ID
                role_id.into(),          // $6 - Role ID
            ],
        );

        // Execute the SQL statement and get the result
        let project_result = db.query_one(stmt).await?;

        // Extract the project details from the query result
        let project = if let Some(row) = project_result {
            projects::Model {
                id: row.try_get("", "id")?,                             // Project ID
                company_id: row.try_get("", "company_id")?,             // Company ID
                title: row.try_get("", "title")?,                       // Project title
                description: row.try_get("", "description")?, // Project description (optional)
                decoration_color: row.try_get("", "decoration_color")?, // Decoration color (optional)
                created_at: row.try_get("", "created_at")?,             // Creation timestamp
                updated_at: row.try_get("", "updated_at")?,             // Update timestamp
            }
        } else {
            return Err(CoreErrors::DatabaseServiceError(
                "Failed to create project".to_string(),
            ));
        };

        // Return the created project
        Ok(project)
    }

    /// Retrieves a user's role and project details for a specific project.
    ///
    /// This function queries both the `user_company` and `user_access` tables to determine the user's role
    /// and their association with the specified project. It returns the user's role and project details if found.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user whose project access is being queried.
    /// * `project_id` - The ID of the project for which access is being checked.
    ///
    /// # Returns
    ///
    /// * `Result<Option<UserProject>, CoreErrors>` - Returns a `UserProject` with role and project details if found,
    ///   or `None` if no association exists. Returns `CoreErrors` if the query fails.
    pub async fn get_user_project(
        db: &DbConn,
        user_id: i32,
        project_id: i32,
    ) -> Result<Option<UserProject>, CoreErrors> {
        // SQL query to retrieve the user's role and project details
        let sql = r#"
        WITH user_role AS (
            -- Check user's role and project details from `user_company`
            SELECT 
                uc.role_id AS role_id,
                r.name AS role_name,
                p.id AS project_id,
                p.company_id AS project_company_id,
                p.title AS project_title,
                p.description AS project_description,
                p.decoration_color AS project_decoration_color
            FROM user_company uc
            JOIN roles r ON uc.role_id = r.id
            JOIN projects p ON uc.company_id = p.company_id
            WHERE uc.user_id = $1 
              AND p.id = $2 
              AND uc.role_id <= 2

            UNION ALL

            -- Check user's role and project details from `user_access`
            SELECT 
                ua.role_id AS role_id,
                r.name AS role_name,
                p.id AS project_id,
                p.company_id AS project_company_id,
                p.title AS project_title,
                p.description AS project_description,
                p.decoration_color AS project_decoration_color
            FROM user_access ua
            JOIN roles r ON ua.role_id = r.id
            JOIN projects p ON ua.project_id = p.id
            WHERE ua.user_id = $1 AND ua.project_id = $2
        )
        SELECT 
            ur.project_id AS project_id,
            ur.project_company_id AS project_company_id,
            ur.project_title AS project_title,
            ur.project_description AS project_description,
            ur.project_decoration_color AS project_decoration_color,
            ur.role_id AS role_id,
            ur.role_name AS role_name
        FROM user_role ur
        LIMIT 1
    "#;

        // Step 1: Prepare the SQL statement with parameters
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            sql,
            [user_id.into(), project_id.into()],
        );

        // Step 2: Execute the query and retrieve the result
        let query_result = UserProjectQueryResult::find_by_statement(stmt)
            .one(db)
            .await?;

        // Step 3: Map the query result into the `UserProject` structure if found
        if let Some(record) = query_result {
            Ok(Some(UserProject {
                id: record.project_id,                             // Project ID
                company_id: record.project_company_id,             // Company ID
                title: record.project_title,                       // Project title
                description: record.project_description,           // Optional project description
                decoration_color: record.project_decoration_color, // Optional decoration color
                user_role: UserProjectRole {
                    id: record.role_id,     // Role ID
                    name: record.role_name, // Role name
                },
            }))
        } else {
            // Step 4: Return `None` if no matching association is found
            Ok(None)
        }
    }

    /// Adds a user to a project with a specified role and access level.
    ///
    /// This function checks if the user already has an entry in the `user_access` table for the specified project.
    /// If no entry exists, it creates a new record associating the user with the project. The user is assigned
    /// the "Manager" role (role ID 3) and limited access.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user to be added to the project.
    /// * `project_id` - The ID of the project to which the user will be added.
    ///
    /// # Returns
    ///
    /// * `Result<user_access::Model, CoreErrors>` - Returns the existing or newly created `user_access` record on success,
    ///   or a `CoreErrors` error if the operation fails.
    pub async fn add_user_to_project(
        db: &DbConn,
        user_id: i32,
        project_id: i32,
    ) -> Result<user_access::Model, CoreErrors> {
        // Step 1: Check if the `user_access` entry already exists
        if let Some(existing_access) = user_access::Entity::find()
            .filter(user_access::Column::UserId.eq(user_id))
            .filter(user_access::Column::ProjectId.eq(project_id))
            .one(db)
            .await?
        {
            // Return the existing record if found
            return Ok(existing_access);
        }

        // Step 2: Create a new `user_access` record with the specified details
        let new_access = user_access::ActiveModel {
            user_id: Set(user_id),                       // Set the user ID
            project_id: Set(Some(project_id)),           // Associate with the specified project
            role_id: Set(Some(3)),                       // Assign role ID 3 (Manager)
            access_level: Set(AccessLevelType::Limited), // Set access level to limited
            ..Default::default()                         // Use default values for other fields
        };

        // Step 3: Insert the new role assignment into the database
        let result = new_access.insert(db).await?;

        // Step 4: Return the successfully created `user_access` record
        Ok(result)
    }

    /// Removes a user from a specified project, optionally based on the requester's role level.
    ///
    /// This function checks if the user has an existing association with the project in the `user_access` table.
    /// If the association exists and the requester's role level permits, it deletes the record. If no association
    /// exists or the requester lacks sufficient permissions, an error is returned.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `user_id` - The ID of the user to be removed from the project.
    /// * `project_id` - The ID of the project from which the user is being removed.
    /// * `request_user_id_lvl` - The role level of the requester. If provided, limits the deletion to associations with a lower or equal role level.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if the user was successfully removed,
    ///   or an error if the user is not associated with the project or lacks permissions.
    pub async fn remove_user_from_project(
        db: &DbConn,
        user_id: i32,
        project_id: i32,
        request_user_id_lvl: Option<i32>,
    ) -> Result<(), CoreErrors> {
        // Step 1: Check if the user has an existing association with the specified project
        let existing_access = match request_user_id_lvl {
            Some(level) => {
                println!("level is Some: {}", level);
                println!("user_id is Some: {}", user_id);
                println!("project_id is Some: {}", project_id);
                // Restrict deletion to associations with a role level less than or equal to the requester's level
                user_access::Entity::find()
                    .filter(user_access::Column::UserId.eq(user_id))
                    .filter(user_access::Column::ProjectId.eq(project_id)) // Corrected `CompanyId` to `ProjectId`
                    .filter(user_access::Column::RoleId.gte(level))
                    .one(db)
                    .await?
            }
            None => {
                println!("request_user_id_lvl is None");
                // Check for any association without role level constraints
                user_access::Entity::find()
                    .filter(user_access::Column::UserId.eq(user_id))
                    .filter(user_access::Column::ProjectId.eq(project_id)) // Corrected `CompanyId` to `ProjectId`
                    .one(db)
                    .await?
            }
        };

        // Step 2: If an association exists, delete it; otherwise, return an error
        match existing_access {
            Some(access) => {
                // Convert the record to an active model for deletion
                let active_role = access.into_active_model();

                // Delete the record from the database
                active_role.delete(db).await?;
            }
            None => {
                // Return an error if no association exists for the user in the specified project
                return Err(CoreErrors::DatabaseServiceError(
                    "User not associated with the specified project".to_string(),
                ));
            }
        };

        // Step 3: Return success if the deletion was successful
        Ok(())
    }

    /// Deletes a specified project from the database if it exists.
    ///
    /// This function checks if the project with the specified ID exists in the database. If found, it deletes the project record.
    /// If the project does not exist, it returns an error indicating the absence of the specified project.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `project_id` - The ID of the project to be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if the project was successfully deleted,
    ///   or a `CoreErrors::DatabaseServiceError` if the project does not exist or another error occurs.
    pub async fn delete_project(db: &DbConn, project_id: i32) -> Result<(), CoreErrors> {
        // Step 1: Attempt to find the project by its ID
        let project = projects::Entity::find_by_id(project_id).one(db).await?;

        // Step 2: If the project exists, delete it; otherwise, return an error
        match project {
            Some(project) => {
                // Convert the project entity into an active model for deletion
                let active_project = project.into_active_model();

                // Attempt to delete the project record from the database
                active_project.delete(db).await?;
            }
            None => {
                // Return an error if the project does not exist
                return Err(CoreErrors::DatabaseServiceError(format!(
                    "Project with ID {} does not exist",
                    project_id
                )));
            }
        };

        // Step 3: Return success if the deletion was successful
        Ok(())
    }

    /// Deletes all user associations from a specified project.
    ///
    /// This function removes all records in the `user_access` table for the given project ID,
    /// effectively removing all users associated with the project.
    ///
    /// # Arguments
    ///
    /// * `db` - A reference to the database connection.
    /// * `project_id` - The ID of the project whose user associations should be deleted.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoreErrors>` - Returns `Ok(())` if the associations were successfully deleted,
    ///   or a `CoreErrors` error if the operation fails.
    pub async fn delete_all_users_from_project(
        db: &DbConn,
        project_id: i32,
    ) -> Result<(), CoreErrors> {
        // Step 1: Attempt to delete all user associations for the specified project
        let _: DeleteResult = user_access::Entity::delete_many()
            .filter(user_access::Column::ProjectId.eq(project_id)) // Filter by project ID
            .exec(db)
            .await?;

        // Step 3: Return success if the deletion was executed without errors
        Ok(())
    }
}
