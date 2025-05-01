import express from "express";
import { fileURLToPath } from "url";
import path from "path";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const app = express();
const port = 3000;

// Serve static files from the test-pages directory
app.use(express.static(path.join(__dirname, "ux", "tests", "test-pages")));

// Serve node_modules for dependencies
app.use(
  "/node_modules",
  express.static(path.join(__dirname, "..", "..", "node_modules")),
);

// Serve the dist directory for compiled code
app.use("/dist", express.static(path.join(__dirname, "dist")));

// Serve the ux directory for direct access to components
app.use("/ux", express.static(path.join(__dirname, "ux")));

// Serve the src directory for access to shared modules
app.use("/src", express.static(path.join(__dirname, "src")));

// Add a route to serve the preact library from ESM CDN
app.get("/preact.js", (req, res) => {
  res.redirect("https://esm.sh/preact@10.20.0");
});

app.get("/preact-compat.js", (req, res) => {
  res.redirect("https://esm.sh/preact@10.20.0/compat");
});

// Default route - redirect to the test page
app.get("/", (req, res) => {
  res.redirect("/circuit-test.html");
});

// Start the server
app.listen(port, () => {
  console.log(`Test server running at http://localhost:${port}`);
});
