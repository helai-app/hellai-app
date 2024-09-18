#!/bin/sh

# Prompt for JWT_SECRET_KEY
read -p "Enter JWT_SECRET_KEY (leave blank to generate randomly): " JWT_SECRET_KEY

if [ -z "$JWT_SECRET_KEY" ]; then
    # Generate a random 1024-character hex string
    JWT_SECRET_KEY=$(openssl rand -hex 512)
fi

# Prompt for POSTGRES_DB with default value
read -p "Enter POSTGRES_DB [core_service_db]: " POSTGRES_DB
POSTGRES_DB=${POSTGRES_DB:-core_service_db}

# Prompt for POSTGRES_USER with default value
read -p "Enter POSTGRES_USER [core_service_db_admin]: " POSTGRES_USER
POSTGRES_USER=${POSTGRES_USER:-core_service_db_admin}

# Prompt for POSTGRES_PASSWORD with default value
read -p "Enter POSTGRES_PASSWORD [123QWEasd]: " POSTGRES_PASSWORD
POSTGRES_PASSWORD=${POSTGRES_PASSWORD:-123QWEasd}

# Generate DATABASE_URL
DATABASE_URL="postgres://${POSTGRES_USER}:${POSTGRES_PASSWORD}@localhost/${POSTGRES_DB}"

# Write variables to .env file
cat > .env <<EOL
JWT_SECRET_KEY=$JWT_SECRET_KEY
POSTGRES_DB=$POSTGRES_DB
POSTGRES_USER=$POSTGRES_USER
POSTGRES_PASSWORD=$POSTGRES_PASSWORD
DATABASE_URL=$DATABASE_URL
EOL

echo "Core service success init."
