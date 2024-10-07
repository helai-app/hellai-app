# Core Service

The Core Service is a backend microservice designed for simple user authentication, data registration, and managing primary user information. Built with **Rust** and **Docker**, it offers robust performance and easy deployment. The service utilizes **gRPC** for efficient, high-performance communication between services.

## Features

- Secure user authentication
- Efficient data registration
- Management of primary user information
- High performance with Rust
- Containerized deployment using Docker
- Inter-service communication via gRPC

## Services Overview

The Core Service provides several key gRPC services to manage projects and users, enabling effective inter-service communication within your system.

### ProjectsService

The `ProjectsService` is responsible for managing project-related tasks:

- **CreateProject**: Creates a new project.
- **AddUserToProject**: Adds a user to an existing project.
- **RemoveUserFromProject**: Removes a user from a project.
- **DeleteProject**: Deletes an existing project.

These RPC methods are used to handle project lifecycle management and manage user assignments in projects.

### UserService

The `UserService` handles user-related operations:

- **AuthenticateWithPassword**: Authenticates users using login and password.
- **RegisterUser**: Registers a new user with login, password, and email.
- **RefreshSessionToken**: Refreshes session tokens for active users.

These methods enable secure user authentication, registration, and session management.

## Installation

Follow these steps to set up and run the Core Service:

1. **Clone the Repository**

   ```bash
   git clone https://github.com/yourusername/core-service.git
   cd core-service
   ```

2. **Generate New Authentication Data**

   Run the installation script to generate new authentication data:

   ```bash
   bash install_core_service.sh
   ```

3. **Start Docker Containers**

   Use Docker Compose to start the necessary services in detached mode:

   ```bash
   docker-compose up -d
   ```

4. **Run the Application**

   Start the Rust application using Cargo:

   ```bash
   cargo run
   ```

## Configuration

Configuration settings can be adjusted in the `.env` file or within the `docker-compose.yml` and `Cargo.toml` files as needed. Common configuration options include:

- **Database settings** (e.g., connection strings, credentials)
- **Authentication settings** (e.g., token expiration time)
- **Service ports** (e.g., the gRPC server port)

Ensure that all required variables are properly set to allow the Core Service to communicate effectively with other parts of your system.

## First-Time Login

To log in for the first time, use the following credentials:

- **Login**: `admin`
- **Password**: `Admin123`

> **Note**: It is highly recommended to change the password after the first login to ensure security.

## Usage Examples

Below are examples of how to interact with the gRPC services.

### Create a Project

To create a new project using `ProjectsService`, send a `CreateProjectRequest` with the project name. This will create a new project and return its details.

```proto
CreateProjectRequest {
  project_name: "My First Project"
}
```

Response:

```proto
CreateProjectResponse {
  project_id: 123,
  project_name: "My First Project"
}
```

### Add User to a Project

To add a user to a project, use the `AddUserToProject` RPC. Send a `UserProjectModificationRequest` with the user ID and project ID.

```proto
UserProjectModificationRequest {
  user_id: 456,
  project_id: 123
}
```

Response:

```proto
ProjectUserInfoResponse {
  user_id: 456,
  user_role: PROJECT_ROLE_USER
}
```

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.

Before submitting changes, please make sure to:

1. Write tests for new functionalities.
2. Format your code using the project's formatting guidelines.
3. Update the documentation if needed.

## Contact

For more information or support, feel free to contact the project maintainers or visit our [GitHub Issues page](https://github.com/helai-app/hellai-app/issues).
