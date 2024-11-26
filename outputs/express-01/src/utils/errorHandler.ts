import createError from "http-errors"; // Importing http-errors to standardize error handling
import express from "express"; // Importing express to use its types

/**
 * Error handling middleware for logging errors to the console
 * and sending standardized responses to the client.
 *
 * @param {Object} err - Error object
 * @param {Object} req - Express request object
 * @param {Object} res - Express response object
 * @param {Function} next - Next middleware function
 */
function errorHandler(
  err: any,
  req: express.Request,
  res: express.Response,
  next: express.NextFunction
) {
  // Log the error stack trace for debugging purposes
  console.error(err.stack);

  // If the error status is defined, use it; otherwise, default to 500
  const statusCode = err.status || 500;

  // Respond to the client with the status code and error message
  res.status(statusCode).json({
    status: "error",
    statusCode,
    message: err.message || "Internal Server Error",
  });
}

/**
 * Middleware to handle 404 errors for routes that are not found.
 *
 * @param {Object} req - Express request object
 * @param {Object} res - Express response object
 * @param {Function} next - Next middleware function
 */
function notFoundHandler(
  req: express.Request,
  res: express.Response,
  next: express.NextFunction
) {
  // Pass a 404 error to the errorHandler if a route is not found
  next(createError(404, "Not Found"));
}

export { errorHandler, notFoundHandler };
