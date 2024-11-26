import express from "express";
import bodyParser from "body-parser";
import userRoutes from "./routes/userRoutes";
import { errorHandler } from "./utils/errorHandler";

const app = express();

// Middleware to parse incoming request bodies
app.use(bodyParser.json());

// Use user routes for handling requests to /api/users
app.use("/api/users", userRoutes);

// Error handling middleware
app.use(errorHandler);

// Export the app for use in server.js
export default app;
