# Express API with User Authentication

## Brief Description

This project is an Express.js API that provides basic user authentication functionality, leveraging the MVC architecture. It's designed as a starting point for developers to implement secure user management in their applications.

## Key Features

- User Authentication: Implements JWT-based authentication to secure user endpoints.
- MVC Structure: Organized using the Model-View-Controller pattern for clean separation of concerns.
- Basic Error Handling: Centralized approach to capture and respond to API errors.

## Quick Start Guide

### Prerequisites

- Node.js and npm installed on your system.
- MongoDB database for storing user information.

### Installation

1. **Clone the Repository**:
   ```bash
   git clone https://github.com/yourusername/express-auth-api.git
   cd express-auth-api
   ```

2. **Install Dependencies**:
   ```bash
   npm install
   ```

3. **Environment Setup**:
   - Create a `.env` file in the root directory and add the following variables:
     ```plaintext
     PORT=3000
     MONGODB_URI=[YOUR_MONGODB_URI]
     JWT_SECRET=[YOUR_JWT_SECRET]
     ```

### Basic Usage

- **Run the Server**:
  ```bash
  npm start
  ```
  or
  ```bash
  node src/server.js
  ```
- The server will start on the specified `PORT` and connect to the MongoDB database.

## Configuration

- **Environment Variables**:
  - `PORT`: Port number for the server.
  - `MONGODB_URI`: Connection string for MongoDB.
  - `JWT_SECRET`: Secret key for JSON Web Tokens.

## Basic Troubleshooting

- **Server Issues**:
  - Ensure all environment variables are set correctly in the `.env` file.
  - Check if MongoDB is running and accessible.
  
Following these instructions will set up the project and get the server running with user authentication ready to be used and further developed.