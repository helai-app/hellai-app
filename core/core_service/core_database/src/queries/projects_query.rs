use core_error::core_errors::CoreErrors;

use sea_orm::prelude::DateTimeWithTimeZone;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DbBackend, DbConn,
    EntityTrait, FromQueryResult, IntoActiveModel, QueryFilter, Set, Statement,
};

use crate::entity::prelude::{Projects, UserAccess};
use crate::entity::projects::{self};
use crate::entity::user_access;

/// Represents a project associated with a user, along with the user's role in that project.
pub struct UserProject {
    pub id: i32,
    pub company_id: i32,
    pub title: String,
    pub description: Option<String>,
    pub decoration_color: Option<String>,
    user_role: UserProjectRole,
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

    /// Creates a new project with the given name and assigns the creator as the owner.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `name` - The name of the new project.
    /// * `user_id` - The ID of the user creating the project, who will be assigned the owner role.
    ///
    /// # Returns
    ///
    /// The created `projects::Model`, or an error of type `CoreErrors`.
    pub async fn create_project(
        db: &DbConn,
        name: String,
        user_id: i32,
    ) -> Result<projects::Model, CoreErrors> {
        // Define the SQL statement using Common Table Expressions (CTEs)
        let sql = r#"
        WITH new_project AS (
            INSERT INTO projects (name)
            VALUES ($1)
            RETURNING id, name
        ),
        inserted_user_project_role AS (
            INSERT INTO user_project_roles (user_id, project_id, project_role_id)
            SELECT $2, id, 1
            FROM new_project
            RETURNING id
        )
        SELECT id, name
        FROM new_project;
    "#;

        // Prepare the SQL statement with parameters
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            sql,
            vec![name.into(), user_id.into()],
        );

        // Execute the SQL statement and get the result
        let project_result = db.query_one(stmt).await?;

        // Extract the project details from the query result
        let project = if let Some(row) = project_result {
            let id: i32 = row.try_get("", "id")?;
            let company_id: i32 = row.try_get("", "company_id")?;
            let title: String = row.try_get("", "title")?;
            let description: Option<String> = row.try_get("", "description")?;
            let decoration_color: Option<String> = row.try_get("", "decoration_color")?;
            let created_at: DateTimeWithTimeZone = row.try_get("", "created_at")?;
            let updated_at: DateTimeWithTimeZone = row.try_get("", "updated_at")?;

            projects::Model {
                id,
                company_id: company_id,
                title: title,
                description: description,
                decoration_color: decoration_color,
                created_at: created_at,
                updated_at: updated_at,
            }
        } else {
            return Err(CoreErrors::DatabaseServiceError(
                "failed_create_project".to_string(),
            ));
        };

        Ok(project)
    }

    /// Deletes a project by its ID.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `project_id` - The ID of the project to delete.
    ///
    /// # Returns
    ///
    /// An empty `Result` on success, or an error of type `CoreErrors`.
    pub async fn delete_project(db: &DbConn, project_id: i32) -> Result<(), CoreErrors> {
        Projects::delete_by_id(project_id).exec(db).await?;
        Ok(())
    }

    /// Adds a user to a project or updates their role if they are already part of the project.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `user_id` - The ID of the user.
    /// * `project_id` - The ID of the project.
    /// * `project_role_id` - The ID of the role to assign to the user in the project.
    ///
    /// # Returns
    ///
    /// The updated or newly created `user_project_roles::Model`, or an error of type `CoreErrors`.
    pub async fn set_user_project_role(
        db: &DbConn,
        user_id: i32,
        project_id: i32,
        project_role_id: i32,
    ) -> Result<user_access::Model, CoreErrors> {
        // Check if the user already has a role in the project
        let existing_role = UserAccess::find()
            .filter(user_access::Column::UserId.eq(user_id))
            .filter(user_access::Column::ProjectId.eq(project_id))
            .one(db)
            .await?;

        let result = match existing_role {
            Some(role) => {
                // Update the existing role
                let mut active_role = role.into_active_model();
                active_role.role_id = Set(Some(project_role_id));
                active_role.update(db).await?
            }
            None => {
                // Create a new role assignment
                let new_role = user_access::ActiveModel {
                    user_id: Set(user_id),
                    project_id: Set(Some(project_id)),
                    role_id: Set(Some(project_role_id)),
                    ..Default::default()
                };
                new_role.insert(db).await?
            }
        };

        Ok(result)
    }

    /// Removes a user from a project by deleting their role assignment.
    ///
    /// # Arguments
    ///
    /// * `db` - The database connection.
    /// * `project_id` - The ID of the project.
    /// * `user_id` - The ID of the user.
    ///
    /// # Returns
    ///
    /// An empty `Result` on success, or an error of type `CoreErrors`.
    pub async fn remove_user_from_project(
        db: &DbConn,
        project_id: i32,
        user_id: i32,
    ) -> Result<(), CoreErrors> {
        // Delete the role assignment from user_project_roles
        let delete_result = UserAccess::delete_many()
            .filter(user_access::Column::UserId.eq(user_id))
            .filter(user_access::Column::ProjectId.eq(project_id))
            .exec(db)
            .await?;

        if delete_result.rows_affected == 0 {
            // Optionally handle the case where no records were deleted
            // e.g., return an error if the user was not part of the project
        }

        Ok(())
    }

    pub async fn get_user_role_in_project(
        db: &DbConn,
        project_id: i32,
        user_id: i32,
    ) -> Result<Option<user_access::Model>, CoreErrors> {
        // Delete the role assignment from user_project_roles
        let user_role = UserAccess::find()
            .filter(user_access::Column::UserId.eq(user_id))
            .filter(user_access::Column::ProjectId.eq(project_id))
            .one(db)
            .await?;

        Ok(user_role)
    }
}
