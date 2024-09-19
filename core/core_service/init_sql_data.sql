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

-- 3. SocialLogins
CREATE TABLE SocialLogins (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    access_token VARCHAR(255),
    refresh_token VARCHAR(255),
    token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    UNIQUE (provider, provider_user_id),
    UNIQUE (user_id, provider)
);

-- 4. GlobalRoles
CREATE TABLE GlobalRoles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description VARCHAR(255)
);

-- 5. UserGlobalRoles
CREATE TABLE UserGlobalRoles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    global_role_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (global_role_id) REFERENCES GlobalRoles(id) ON DELETE CASCADE,
    UNIQUE (user_id, global_role_id)
);

-- 6. Companies
CREATE TABLE Companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL
);

-- 7. UserCompanies
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

-- 8. CompanyRoles
CREATE TABLE CompanyRoles (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    FOREIGN KEY (company_id) REFERENCES Companies(id) ON DELETE CASCADE,
    UNIQUE (company_id, name)
);

-- 9. UserCompanyRoles
CREATE TABLE UserCompanyRoles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    company_role_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_role_id) REFERENCES CompanyRoles(id) ON DELETE CASCADE,
    UNIQUE (user_id, company_role_id)
);


Certainly! Below is the SQL code to create the specified database schema in PostgreSQL:

```sql
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

-- 3. SocialLogins
CREATE TABLE SocialLogins (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255) NOT NULL,
    access_token VARCHAR(255),
    refresh_token VARCHAR(255),
    token_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    UNIQUE (provider, provider_user_id),
    UNIQUE (user_id, provider)
);

-- 4. GlobalRoles
CREATE TABLE GlobalRoles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description VARCHAR(255)
);

-- 5. UserGlobalRoles
CREATE TABLE UserGlobalRoles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    global_role_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (global_role_id) REFERENCES GlobalRoles(id) ON DELETE CASCADE,
    UNIQUE (user_id, global_role_id)
);

-- 6. Companies
CREATE TABLE Companies (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) UNIQUE NOT NULL
);

-- 7. UserCompanies
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

-- 8. CompanyRoles
CREATE TABLE CompanyRoles (
    id SERIAL PRIMARY KEY,
    company_id INTEGER NOT NULL,
    name VARCHAR(255) NOT NULL,
    description VARCHAR(255),
    FOREIGN KEY (company_id) REFERENCES Companies(id) ON DELETE CASCADE,
    UNIQUE (company_id, name)
);

-- 9. UserCompanyRoles
CREATE TABLE UserCompanyRoles (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL,
    company_role_id INTEGER NOT NULL,
    FOREIGN KEY (user_id) REFERENCES Users(id) ON DELETE CASCADE,
    FOREIGN KEY (company_role_id) REFERENCES CompanyRoles(id) ON DELETE CASCADE,
    UNIQUE (user_id, company_role_id)
);
```

-- ### **Additional Notes:**

-- - **Auto-updating `updated_at`:**
--   - PostgreSQL does not automatically update the `updated_at` field on record updates. To achieve this, you can create a trigger function:

--     ```sql
--     CREATE OR REPLACE FUNCTION update_updated_at_column()
--     RETURNS TRIGGER AS $$
--     BEGIN
--       NEW.updated_at = NOW();
--       RETURN NEW;
--     END;
--     $$ LANGUAGE plpgsql;
--     ```

--   - Then, add triggers to the tables that need auto-updating `updated_at`:

--     ```sql
--     CREATE TRIGGER update_users_updated_at
--     BEFORE UPDATE ON Users
--     FOR EACH ROW
--     EXECUTE PROCEDURE update_updated_at_column();

--     CREATE TRIGGER update_passwords_updated_at
--     BEFORE UPDATE ON Passwords
--     FOR EACH ROW
--     EXECUTE PROCEDURE update_updated_at_column();

--     -- Repeat for other tables as needed
--     ```

-- - **Indexing:**
--   - Consider adding indexes on foreign key columns to improve query performance.

--     ```sql
--     CREATE INDEX idx_passwords_user_id ON Passwords(user_id);
--     CREATE INDEX idx_sociallogins_user_id ON SocialLogins(user_id);
--     CREATE INDEX idx_userglobalroles_user_id ON UserGlobalRoles(user_id);
--     CREATE INDEX idx_userglobalroles_global_role_id ON UserGlobalRoles(global_role_id);
--     CREATE INDEX idx_usercompanies_user_id ON UserCompanies(user_id);
--     CREATE INDEX idx_usercompanies_company_id ON UserCompanies(company_id);
--     CREATE INDEX idx_companyroles_company_id ON CompanyRoles(company_id);
--     CREATE INDEX idx_usercompanyroles_user_id ON UserCompanyRoles(user_id);
--     CREATE INDEX idx_usercompanyroles_company_role_id ON UserCompanyRoles(company_role_id);
--     ```

-- - **Character Encoding:**
--   - Ensure your database and tables are set up with the appropriate character encoding (e.g., UTF-8) to support international characters.
