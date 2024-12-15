# Documentation: Simplified Access Control System for CRM Project

## Table of Contents

- [Introduction](#introduction)
- [Simplified Access Control System](#simplified-access-control-system)
  - [Hierarchy Levels](#hierarchy-levels)
  - [Access Inheritance](#access-inheritance)
  - [Notes Access](#notes-access)
- [Updated SQL Schema Overview](#updated-sql-schema-overview)
- [Example Scenarios](#example-scenarios)
  - [Adding a User to a Company](#adding-a-user-to-a-company)
  - [Adding a User to a Project](#adding-a-user-to-a-project)
  - [Adding a User to a Task](#adding-a-user-to-a-task)
  - [Adding a User to a Subtask](#adding-a-user-to-a-subtask)
- [Application Logic Considerations](#application-logic-considerations)
  - [Permission Checks](#permission-checks)
  - [Access Inheritance Logic](#access-inheritance-logic)
  - [Role-Based Permissions](#role-based-permissions)
- [Advantages of the Simplified System](#advantages-of-the-simplified-system)
- [Final Notes](#final-notes)

## Introduction

This document outlines a simplified access control system for a Customer Relationship Management (CRM) project. The system is designed to manage user permissions across various hierarchical levels—Company, Project, Task, and Subtask—ensuring that users have appropriate access based on their roles and assigned entities. The goal is to maintain a balance between flexibility and simplicity, allowing for efficient permission management without unnecessary complexity.

## Simplified Access Control System

### Hierarchy Levels

The access control system is structured around four primary hierarchy levels:

1. **Company (Firm)**
2. **Project**
3. **Task**
4. **Subtask**

### Access Inheritance

Permissions granted at a higher level automatically grant access to all lower levels within the same hierarchy, unless explicitly restricted.

- **Company Access**: Grants access to all projects, tasks, and subtasks within the company.
- **Project Access**: Grants access to all tasks and subtasks within the project.
- **Task Access**: Grants access to the task and its subtasks.
- **Subtask Access**: Grants access only to the specific subtask.

### Notes Access

- **Inheritance**: Access to notes is determined by the entity they are associated with (Company, Project, Task, Subtask).
- **Personal Notes**: Users can create personal notes not linked to any entity. These notes are only accessible by the note's creator.

## Updated SQL Schema Overview

The database schema is designed to reflect the simplified access control system. Key components include:

- **Users**: Stores user information.
- **Companies**: Stores company (firm) information.
- **Roles**: Defines user roles with hierarchical levels.
- **UserCompany**: Associates users with companies and roles.
- **Projects**: Manages projects within companies.
- **Tasks**: Manages tasks within projects.
- **Subtasks**: Manages subtasks within tasks.
- **Notes**: Stores notes associated with users or entities.
- **KnowledgeBase**: Manages knowledge base resources with access control.
- **UserAccess**: Central table for managing user permissions across hierarchy levels.

**Note**: The full SQL schema is defined separately and includes all tables, fields, constraints, and indexes necessary for implementing the system.

## Example Scenarios

### Adding a User to a Company

**Objective**: Grant a user access to all data within a company.

**Implementation**:

- Insert a record into the `UserAccess` table with the `user_id` and `company_id` fields populated.
- Set the `access_level` as required (e.g., `'full'`).

**Result**:

- The user gains access to the company and, by inheritance, to all projects, tasks, and subtasks under that company.
- The user can access notes and knowledge base resources associated with the company.

### Adding a User to a Project

**Objective**: Grant a user access to a specific project and all its tasks and subtasks.

**Implementation**:

- Insert a record into the `UserAccess` table with the `user_id` and `project_id` fields populated.
- Set the `access_level` as required.

**Result**:

- The user gains access to the project and, by inheritance, to all tasks and subtasks within that project.
- The user can access notes associated with the project and its tasks/subtasks.

### Adding a User to a Task

**Objective**: Grant a user access to a specific task and its subtasks.

**Implementation**:

- Insert a record into the `UserAccess` table with the `user_id` and `task_id` fields populated.
- Set the `access_level` as required.

**Result**:

- The user gains access to the task and its subtasks.
- The user can access notes associated with the task and subtasks.

### Adding a User to a Subtask

**Objective**: Grant a user access to a specific subtask only.

**Implementation**:

- Insert a record into the `UserAccess` table with the `user_id` and `subtask_id` fields populated.
- Set the `access_level` as required.

**Result**:

- The user gains access only to the specified subtask.
- The user can access notes associated with that subtask.

## Application Logic Considerations

### Permission Checks

When a user attempts to access an entity, the application should perform the following checks:

1. **Direct Permission**: Check the `UserAccess` table for explicit permission at the entity's level.
2. **Inherited Permission**: If no direct permission exists, check for permissions at higher levels (e.g., task, project, company) that would grant inherited access.
3. **Role-Based Actions**: Determine what actions the user can perform based on their role defined in the `UserCompany` table.

### Access Inheritance Logic

- **Company Level**: If a user has access at the company level, they automatically have access to all underlying projects, tasks, and subtasks.
- **Project Level**: Access at the project level grants permissions to all tasks and subtasks within that project.
- **Task Level**: Access at the task level includes all subtasks under that task.
- **Subtask Level**: Access is limited to the specific subtask.

**Implementation**:

- The application logic should recursively check for permissions from the specific entity up to the company level.
- Efficient querying can be achieved by ordering checks from the most specific to the most general level.

### Role-Based Permissions

Roles define the scope of actions a user can perform within their access level:

- **Administrator**: Full control over all entities and permissions within the company.
- **Manager**: Can manage projects, tasks, and users within assigned projects.
- **Employee**: Limited to viewing and updating assigned tasks and subtasks.

**Role Assignment**:

- Roles are assigned per company in the `UserCompany` table.
- Role hierarchy is defined in the `Roles` table, allowing for structured permission escalation.

## Advantages of the Simplified System

- **Ease of Understanding**: A clear hierarchy and inheritance model simplifies permission management.
- **Reduced Complexity**: Fewer tables and relationships make the system more maintainable.
- **Efficient Permission Checks**: Simplified logic for checking permissions improves application performance.
- **Flexibility**: Users can be granted access at any level, accommodating various organizational structures.
- **Scalability**: The system can easily adapt to additional hierarchy levels or entities if needed.

## Final Notes

The simplified access control system provides a balanced approach to managing user permissions within a CRM project. By leveraging hierarchical access and inheritance, the system reduces complexity while maintaining precise control over user permissions. The design ensures that users have appropriate access based on their assigned roles and associated entities, enhancing both security and usability.

**Recommendations**:

- **Testing**: Thoroughly test permission scenarios to ensure the inheritance logic works as intended.
- **Auditing**: Implement logging of access and permission changes for security auditing.
- **Documentation**: Maintain up-to-date documentation for developers and administrators to understand the permission system.

**Next Steps**:

- Implement the application logic for permission checks as outlined.
- Develop user interfaces for managing roles and permissions at different hierarchy levels.
- Continuously review and refine the access control system based on user feedback and organizational needs.