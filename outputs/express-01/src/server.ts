import app from "./app";
import config from "./config/config";

// Get the port from configuration or default to 3000
const PORT = config.port || 3000;

// Start the server and listen on the specified port
app.listen(PORT, () => {
  console.log(`Server is running on port ${PORT}`);
});
