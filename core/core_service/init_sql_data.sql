-- 1. Users
CREATE TABLE Users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 2. Passwords
CREATE TABLE Passwords (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE
);

-- 3. GlobalRoles
CREATE TABLE GlobalRoles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description VARCHAR(255)
);

-- 4. UserGlobalRoles
CREATE TABLE UserGlobalRoles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    global_role_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (global_role_id) REFERENCES GlobalRoles(id) ON DELETE CASCADE,
    UNIQUE (user_id, global_role_id)
);

-- 5. Projects
CREATE TABLE Projects (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL
);

-- 6. UserProjects
CREATE TABLE UserProjects (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES Projects(id) ON DELETE CASCADE,
    UNIQUE (user_id, project_id)
);

-- 7. ProjectRoles (All roles within a project)
CREATE TABLE ProjectRoles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL,
    description VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- 8. UserProjectRoles (User roles in a project)
CREATE TABLE UserProjectRoles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    project_id INTEGER NOT NULL,
    project_role_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES Projects(id) ON DELETE CASCADE,
    FOREIGN KEY (project_role_id) REFERENCES ProjectRoles(id) ON DELETE CASCADE,
    UNIQUE (user_id, project_id, project_role_id)
);

-- ### Additional Notes ###

-- 9. Auto-updating `updated_at`
-- Create a trigger function for auto-updating `updated_at` fields.
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
  NEW.updated_at = NOW();
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- 10. Add triggers to tables for auto-updating `updated_at`
CREATE TRIGGER update_users_updated_at
BEFORE UPDATE ON Users
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_passwords_updated_at
BEFORE UPDATE ON Passwords
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_userprojects_updated_at
BEFORE UPDATE ON UserProjects
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_projectroles_updated_at
BEFORE UPDATE ON ProjectRoles
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_userprojectroles_updated_at
BEFORE UPDATE ON UserProjectRoles
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

-- 11. Indexing
-- Improve performance by adding indexes on foreign key columns.
CREATE INDEX idx_passwords_user_id ON Passwords(user_id);
CREATE INDEX idx_userglobalroles_user_id ON UserGlobalRoles(user_id);
CREATE INDEX idx_userglobalroles_global_role_id ON UserGlobalRoles(global_role_id);
CREATE INDEX idx_userprojects_user_id ON UserProjects(user_id);
CREATE INDEX idx_userprojects_project_id ON UserProjects(project_id);
CREATE INDEX idx_userprojectroles_user_id ON UserProjectRoles(user_id);
CREATE INDEX idx_userprojectroles_project_id ON UserProjectRoles(project_id);
CREATE INDEX idx_userprojectroles_project_role_id ON UserProjectRoles(project_role_id);

-- 12. Character Encoding
-- Ensure that the database supports UTF-8 encoding.
