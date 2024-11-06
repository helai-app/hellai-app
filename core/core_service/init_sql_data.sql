-- Define ENUM types for access levels and task statuses
CREATE TYPE access_level_type AS ENUM ('full', 'limited', 'restricted');
CREATE TYPE task_status_type AS ENUM ('pending', 'in_progress', 'completed');

-- Table for storing user information
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    login VARCHAR(50) NOT NULL UNIQUE,
    user_name VARCHAR(50) NOT NULL,
    email VARCHAR(100) NOT NULL UNIQUE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC',
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
);

-- Table for storing passwords
CREATE TABLE Passwords (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC',
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
);
CREATE INDEX idx_passwords_user_id ON Passwords(user_id);

-- Table for storing company information
CREATE TABLE Companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    name_alias VARCHAR(100) NOT NULL UNIQUE CHECK (name_alias ~ '^[a-z]+$'),
    description VARCHAR(500),
    contact_info VARCHAR(250)
);

-- Table for defining roles with hierarchy and access permissions
CREATE TABLE Roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE,
    description VARCHAR(250),
    parent_role_id INTEGER REFERENCES Roles(id),
    level INTEGER NOT NULL CHECK (level > 0)
);

-- Table for linking users to companies with specific roles
CREATE TABLE UserCompany (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    company_id INTEGER NOT NULL REFERENCES Companies(id) ON DELETE CASCADE,
    role_id INTEGER NOT NULL REFERENCES Roles(id),
    access_level access_level_type NOT NULL
);
CREATE UNIQUE INDEX idx_usercompany_user_company ON UserCompany(user_id, company_id);

-- Table for projects within companies
CREATE TABLE Projects (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES Companies(id) ON DELETE CASCADE,
    title VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    decoration_color VARCHAR(7) CHECK (decoration_color ~ '^#[0-9A-Fa-f]{6}$'),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC',
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
);
CREATE INDEX idx_projects_company_id ON Projects(company_id);

-- Table for tasks within projects
CREATE TABLE Tasks (
    id SERIAL PRIMARY KEY,
    project_id INTEGER NOT NULL REFERENCES Projects(id) ON DELETE CASCADE,
    assigned_to INTEGER REFERENCES Users(id),
    status task_status_type NOT NULL DEFAULT 'pending',
    title VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    priority VARCHAR(20),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC',
    due_date TIMESTAMPTZ
);
CREATE INDEX idx_tasks_project_id ON Tasks(project_id);
CREATE INDEX idx_tasks_assigned_to ON Tasks(assigned_to);

-- Table for subtasks within tasks
CREATE TABLE Subtasks (
    id SERIAL PRIMARY KEY,
    task_id INTEGER NOT NULL REFERENCES Tasks(id) ON DELETE CASCADE,
    assigned_to INTEGER REFERENCES Users(id),
    status task_status_type NOT NULL DEFAULT 'pending',
    title VARCHAR(100) NOT NULL,
    description VARCHAR(500),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC',
    due_date TIMESTAMPTZ
);
CREATE INDEX idx_subtasks_task_id ON Subtasks(task_id);
CREATE INDEX idx_subtasks_assigned_to ON Subtasks(assigned_to);

-- Table for notes with hierarchical access
CREATE TABLE Notes (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES Users(id),
    company_id INTEGER REFERENCES Companies(id) ON DELETE CASCADE,
    project_id INTEGER REFERENCES Projects(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES Tasks(id) ON DELETE CASCADE,
    subtask_id INTEGER REFERENCES Subtasks(id) ON DELETE CASCADE,
    content VARCHAR(1000) NOT NULL,
    tags VARCHAR(250),
    decoration_color VARCHAR(7) CHECK (decoration_color ~ '^#[0-9A-Fa-f]{6}$'),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
);
-- Ensure that notes are linked to only one entity or none (for personal notes)
ALTER TABLE Notes ADD CONSTRAINT chk_notes_entity CHECK (
    (company_id IS NOT NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NULL) OR
    (company_id IS NULL AND project_id IS NOT NULL AND task_id IS NULL AND subtask_id IS NULL) OR
    (company_id IS NULL AND project_id IS NULL AND task_id IS NOT NULL AND subtask_id IS NULL) OR
    (company_id IS NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NOT NULL) OR
    (company_id IS NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NULL)
);
CREATE INDEX idx_notes_user_id ON Notes(user_id);
CREATE INDEX idx_notes_company_id ON Notes(company_id);
CREATE INDEX idx_notes_project_id ON Notes(project_id);
CREATE INDEX idx_notes_task_id ON Notes(task_id);
CREATE INDEX idx_notes_subtask_id ON Notes(subtask_id);

-- Table for the company's knowledge base resources
CREATE TABLE KnowledgeBase (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL REFERENCES Companies(id) ON DELETE CASCADE,
    project_id INTEGER REFERENCES Projects(id) ON DELETE SET NULL,
    title VARCHAR(100) NOT NULL,
    content VARCHAR(1000),
    access_level access_level_type NOT NULL,
    role_id INTEGER REFERENCES Roles(id)
);
CREATE INDEX idx_knowledgebase_company_id ON KnowledgeBase(company_id);
CREATE INDEX idx_knowledgebase_project_id ON KnowledgeBase(project_id);
CREATE INDEX idx_knowledgebase_role_id ON KnowledgeBase(role_id);

-- Simplified permissions table with roles
CREATE TABLE UserAccess (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES Users(id) ON DELETE CASCADE,
    company_id INTEGER REFERENCES Companies(id) ON DELETE CASCADE,
    project_id INTEGER REFERENCES Projects(id) ON DELETE CASCADE,
    task_id INTEGER REFERENCES Tasks(id) ON DELETE CASCADE,
    subtask_id INTEGER REFERENCES Subtasks(id) ON DELETE CASCADE,
    role_id INTEGER REFERENCES Roles(id),
    access_level access_level_type NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP AT TIME ZONE 'UTC'
);
-- Ensure only one of company_id, project_id, task_id, or subtask_id is NOT NULL
ALTER TABLE UserAccess ADD CONSTRAINT chk_useraccess_level CHECK (
    (company_id IS NOT NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NULL) OR
    (company_id IS NULL AND project_id IS NOT NULL AND task_id IS NULL AND subtask_id IS NULL) OR
    (company_id IS NULL AND project_id IS NULL AND task_id IS NOT NULL AND subtask_id IS NULL) OR
    (company_id IS NULL AND project_id IS NULL AND task_id IS NULL AND subtask_id IS NOT NULL)
);
-- Update unique index to include role_id
CREATE UNIQUE INDEX idx_useraccess_unique ON UserAccess(
    user_id,
    COALESCE(company_id, 0),
    COALESCE(project_id, 0),
    COALESCE(task_id, 0),
    COALESCE(subtask_id, 0),
    COALESCE(role_id, 0)
);
-- Create an index on role_id for optimized querying by role
CREATE INDEX idx_useraccess_role_id ON UserAccess(role_id);
CREATE INDEX idx_useraccess_user_id ON UserAccess(user_id);
