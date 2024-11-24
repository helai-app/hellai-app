# Core Service 🚀

The **Core Service** is a backend microservice designed for seamless **user authentication**, **data registration**, and **primary information management**. It is implemented in **Rust** and supports **gRPC** communication for efficient service interaction. The service is containerized with **Docker** for streamlined deployment.

---

## ✨ Features

- 🔒 Secure user authentication
- 📝 Efficient data registration
- 📊 Comprehensive company and project management
- ⚡ High performance with Rust
- 🐳 Containerized deployment using Docker
- 🌐 Inter-service communication via gRPC

---

## 🛠️ gRPC Services Overview

The Core Service exposes a wide range of **gRPC APIs** for managing companies, projects, tasks, notes, and users. Here's an overview:

### 🏢 **CompaniesService**

Manage company-level operations:

- **CreateCompany**: 🏢 Create a new company.
- **AddUserToCompany**: ➕ Assign a user to a company with a specific role.
- **RemoveUserFromCompany**: ❌ Remove a user from a company.
- **DeleteCompany**: 🗑️ Delete an existing company.

---

### 📂 **ProjectsService**

Handle project-level operations:

- **CreateProject**: 🛠️ Create a new project.
- **AddUserToProject**: 👥 Add a user to a project.
- **RemoveUserFromProject**: ❌ Remove a user from a project.
- **DeleteProject**: 🗑️ Delete a project.

---

### ✅ **TasksService**

Organize and manage tasks within projects:

- **CreateTask**: 📝 Create a new task.
- **AddUserToTask**: 👥 Add a user to a task.
- **RemoveUserFromTask**: ❌ Remove a user from a task.
- **DeleteTask**: 🗑️ Delete a task.

---

### 📝 **NotesService**

Capture notes related to companies, projects, or tasks:

- **CreateNote**: 🖊️ Add a new note.
- **DeleteNote**: 🗑️ Delete an existing note.

---

### 🧑‍💻 **UserService**

Manage users and their session data:

- **AuthenticateWithPassword**: 🔑 Authenticate users via login and password.
- **RegisterUser**: 🛠️ Register a new user with credentials and email.
- **RefreshSessionToken**: 🔄 Refresh session tokens.
- **GetUserData**: 📋 Fetch user details and associated company/projects.

---

## 📝 Proto Details

The Core Service defines the following protobuf files for precise API contracts:

1. **CompaniesService Protos**: Manage company operations and user relationships.
2. **ProjectsService Protos**: Handle project-level CRUD operations and user assignments.
3. **TasksService Protos**: Organize tasks and manage user relationships at the task level.
4. **NotesService Protos**: Facilitate note-taking features linked to projects or tasks.
5. **UserService Protos**: Support user authentication, registration, and session management.
6. **DefaultService Protos**: Contain shared enums, messages, and response types.

Detailed proto definitions are available in the repository.

---

## 🚀 Installation

### 1️⃣ **Clone the Repository**
```bash
git clone https://github.com/yourusername/core-service.git
cd core-service
```

### 2️⃣ **Generate Authentication Data**
```bash
bash install_core_service.sh
```

### 3️⃣ **Start Services**
```bash
docker-compose up -d
```

### 4️⃣ **Run the Application**
```bash
cargo run
```

---

## 🔐 First-Time Login

Use the following credentials for the initial setup:

- **Login**: `admin`
- **Password**: `Admin123`

> **⚠️ Note**: Change the password immediately after logging in for security.

---

## 🌐 Contributing

Contributions are welcome! Follow these steps:

1. Write tests for new features.
2. Format code using project guidelines.
3. Update documentation as needed.

Submit a pull request or open an issue on our [GitHub page](https://github.com/helai-app/hellai-app/issues).

---

## 📞 Contact

For support, visit the [GitHub Issues page](https://github.com/helai-app/hellai-app/issues). 

---

### 🚀 Empower your systems with **Core Service**! 🌟