// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/*
Currently the Azure Quantum data-plane doesn't have CORS configured to allow all origins.
Until that is fixed, this script acts as a proxy between VS Code and Azure Quantum.
*/

import { createServer } from "node:http";

const server = createServer(async (req, res) => {
  if (req.method === "OPTIONS") {
    // Send cors stuff
    res.setHeader(
      "access-control-allow-headers",
      "authorization,content-type,x-proxy-to,x-ms-version,x-ms-date"
    );
    res.setHeader(
      "access-control-allow-methods",
      "GET,POST,PUT,DELETE,PATCH,OPTIONS,HEAD"
    );
    res.setHeader("access-control-allow-origin", "*");
    res.setHeader("access-control-max-age", "86400");
    res.setHeader("cache-control", "public, max-age=3110400, immutable");
    res.setHeader("vary", "Access-Control-Allow-Headers");
    res.end();
  } else if (req.method === "GET" || req.method === "POST") {
    // Proxy the request
    const token = req.headers["authorization"]?.substring(7);
    const target = req.headers["x-proxy-to"];
    const path = req.url;

    let body = undefined;
    if (req.method === "POST") {
      // Read the body
      body = await new Promise((resolve) => {
        let data = "";
        req.on("data", (chunk) => {
          data += chunk;
        });
        req.on("end", () => {
          resolve(data);
        });
      });
    }

    const reqHeaders = [];
    if (token) reqHeaders.push(["Authorization", `Bearer ${token}`]);
    if (req.headers["content-type"])
      reqHeaders.push(["Content-type", req.headers["content-type"]]);
    if (req.headers["x-ms-version"])
      reqHeaders.push(["x-ms-version", req.headers["x-ms-version"]]);
    if (req.headers["x-ms-date"])
      reqHeaders.push(["x-ms-date", req.headers["x-ms-date"]]);

    // Fetch from the origin, then return the payload
    const response = await fetch(`${target}${path}`, {
      method: req.method,
      body,
      headers: reqHeaders,
    });

    if (!response.ok) {
      res.statusCode = response.statusCode;
      console.error("Response was an error");
      res.end();
    } else {
      res.setHeader("Content-type", response.headers.get("content-type"));
      res.setHeader("access-control-allow-origin", "*");
      res.write(await response.text());
      res.end();
    }
  } else {
    console.error("Unrecognized method: ", req.method);
  }
});

server.listen(5555);
