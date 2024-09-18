# Core Service

The Core Service is a backend microservice designed for simple user authentication, data registration, and managing primary user information. Built with Rust and Docker, it offers robust performance and easy deployment. The service utilizes **gRPC** for efficient, high-performance communication between services.

## Features

- Secure user authentication
- Efficient data registration
- Management of primary user information
- High performance with Rust
- Containerized deployment using Docker
- Inter-service communication via gRPC

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

Configuration settings can be adjusted in the `.env` file or within the `docker-compose.yml` and `Cargo.toml` files as needed.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any enhancements or bug fixes.