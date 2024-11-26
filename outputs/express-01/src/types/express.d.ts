// express.d.ts
import { Request } from "express";
import { IUser } from "../models/userModel"; // Import the IUser interface

declare global {
  namespace Express {
    interface Request {
      user?: IUser; // Update the type to IUser
    }
  }
}
