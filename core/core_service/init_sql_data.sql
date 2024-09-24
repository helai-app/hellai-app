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
    salt VARCHAR(255) NOT NULL,
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

-- 5. Companies
CREATE TABLE Companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL
);

-- 6. UserCompanies
CREATE TABLE UserCompanies (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    company_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_id) REFERENCES Companies(id) ON DELETE CASCADE,
    UNIQUE (user_id, company_id)
);

-- 7. CompanyRoles (All roles within a company)
CREATE TABLE CompanyRoles (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (company_id) REFERENCES Companies(id) ON DELETE CASCADE,
    UNIQUE (company_id, name)
);

-- 8. UserCompanyRoles (User roles in a company)
CREATE TABLE UserCompanyRoles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    company_id INTEGER NOT NULL,
    company_role_id INTEGER NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_id) REFERENCES Companies(id) ON DELETE CASCADE,
    FOREIGN KEY (company_role_id) REFERENCES CompanyRoles(id) ON DELETE CASCADE,
    UNIQUE (user_id, company_id, company_role_id)
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

-- Repeat for other tables as needed
CREATE TRIGGER update_usercompanies_updated_at
BEFORE UPDATE ON UserCompanies
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_companyroles_updated_at
BEFORE UPDATE ON CompanyRoles
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

CREATE TRIGGER update_usercompanyroles_updated_at
BEFORE UPDATE ON UserCompanyRoles
FOR EACH ROW
EXECUTE PROCEDURE update_updated_at_column();

-- 11. Indexing
-- Improve performance by adding indexes on foreign key columns.
CREATE INDEX idx_passwords_user_id ON Passwords(user_id);
CREATE INDEX idx_userglobalroles_user_id ON UserGlobalRoles(user_id);
CREATE INDEX idx_userglobalroles_global_role_id ON UserGlobalRoles(global_role_id);
CREATE INDEX idx_usercompanies_user_id ON UserCompanies(user_id);
CREATE INDEX idx_usercompanies_company_id ON UserCompanies(company_id);
CREATE INDEX idx_companyroles_company_id ON CompanyRoles(company_id);
CREATE INDEX idx_usercompanyroles_user_id ON UserCompanyRoles(user_id);
CREATE INDEX idx_usercompanyroles_company_id ON UserCompanyRoles(company_id);
CREATE INDEX idx_usercompanyroles_company_role_id ON UserCompanyRoles(company_role_id);

-- 12. Character Encoding
-- Ensure that the database supports UTF-8 encoding.
