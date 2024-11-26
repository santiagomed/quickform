import dotenv from "dotenv";
import assert from "assert";

// Load environment variables from the .env file
dotenv.config();

// Validate required environment variables
assert(process.env.PORT, "PORT environment variable is required");
assert(process.env.MONGODB_URI, "MONGODB_URI environment variable is required");
assert(process.env.JWT_SECRET, "JWT_SECRET environment variable is required");

interface Config {
  port: number;
  mongodbUri: string;
  jwtSecret: string;
  domain: string;
}

const config: Config = {
  // Port number for the server to listen on
  port: Number(process.env.PORT) || 3000,

  // MongoDB connection URI
  mongodbUri: process.env.MONGODB_URI || "mongodb://localhost:27017/myapp",

  // Secret key for JWT authentication
  jwtSecret: process.env.JWT_SECRET || "your_secret_key",

  // Domain for the application
  domain: process.env.DOMAIN || "http://localhost:8080",
};

export default config;
