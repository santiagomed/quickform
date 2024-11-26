import express from "express";
import * as userController from "../controllers/userController";
import authMiddleware from "../middleware/authMiddleware";

const router = express.Router();

// Route to register a new user
router.post("/register", userController.registerUser);

// Route to login a user and receive a JWT token
router.post("/login", userController.loginUser);

// Protected route to get user profile, requires authentication
router.get("/profile", authMiddleware, userController.getUserProfile);

// Protected route to update user profile, requires authentication
router.put("/profile", authMiddleware, userController.updateUserProfile);

// Protected route to delete user account, requires authentication
router.delete("/profile", authMiddleware, userController.deleteUser);

export default router;
