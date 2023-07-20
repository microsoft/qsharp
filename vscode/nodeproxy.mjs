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
      "authorization,content-type,x-proxy-to"
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
  } else if (req.method === "GET") {
    // Proxy the request
    const token = req.headers["authorization"]?.substring(7);
    const target = req.headers["x-proxy-to"];
    const path = req.url;

    // Fetch from the origin, then return the payload
    const response = await fetch(`${target}${path}`, {
      headers: [
        ["Authorization", `Bearer ${token}`],
        ["Content-Type", "application/json"],
      ],
    });

    if (!response.ok) {
      res.statusCode = response.statusCode;
      console.error("Response was an error");
      res.end();
    } else {
      res.setHeader("Content-type", "application/json");
      res.setHeader("access-control-allow-origin", "*");
      res.write(await response.text());
      res.end();
    }
  } else {
    console.error("Unrecognized method: ", req.method);
  }
});

server.listen(5555);
