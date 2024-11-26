import jwt from "jsonwebtoken";
import User from "../models/userModel";
import { Request, Response, NextFunction } from "express";
import config from "../config/config";

// Middleware to protect routes that require authentication
const authMiddleware = async (
  req: Request,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const token = req.header("Authorization")?.replace("Bearer ", "");
    if (!token) {
      res
        .status(401)
        .json({ error: "No token provided, authorization denied" });
      return;
    }

    // Verify the token using the application's secret key
    const decoded = jwt.verify(token, config.jwtSecret) as { id: string };
    const user = await User.findOne({ _id: decoded.id });

    if (!user) {
      res.status(401).json({ error: "Unauthorized, user not found" });
      return;
    }

    // Attach the user to the request object
    req.user = user;
    next();
  } catch (err) {
    res.status(401).json({ error: "Token is not valid" });
  }
};

export default authMiddleware;
