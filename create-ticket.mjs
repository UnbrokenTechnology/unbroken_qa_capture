#!/usr/bin/env node

import { readFileSync, existsSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";

const __dirname = dirname(fileURLToPath(import.meta.url));

// Load API key from .env
const envPath = resolve(__dirname, ".env");
let apiKey = null;

if (existsSync(envPath)) {
  const envContent = readFileSync(envPath, "utf-8");
  apiKey = envContent
    .split("\n")
    .find((line) => line.startsWith("LINEAR_API_KEY="))
    ?.split("=")
    .slice(1)
    .join("=")
    .trim();
}

// Check for dry-run mode
const isDryRun = !apiKey || process.argv.includes("--dry-run");

if (isDryRun && !process.argv.includes("--dry-run")) {
  console.log(JSON.stringify({
    success: false,
    error: "LINEAR_API_KEY not found in .env - running in dry-run mode",
    dryRun: true,
  }));
}

const TEAM_ID = "44c86ac8-cb80-4302-9d81-a0a350b2c352";

// Read JSON input from command line argument
const arg = process.argv.find((a, i) => i > 1 && !a.startsWith("--"));

if (!arg) {
  console.error("ERROR: Pass ticket JSON as a command line argument");
  console.error('Usage: node create-ticket.mjs \'{"title":"...","description":"..."}\'');
  console.error('       node create-ticket.mjs \'{"title":"...","description":"..."}\' --dry-run');
  process.exit(1);
}

const input = JSON.parse(arg);

const {
  title,
  description,
  priority = 0,
  assigneeId,
  labelIds = [],
  stateId = "aa635d13-f2bb-48f2-a395-2fd15e0b0441", // Backlog
} = input;

if (!title || !description) {
  console.error("ERROR: title and description are required");
  process.exit(1);
}

// In dry-run mode, output what would be created
if (isDryRun) {
  console.log(JSON.stringify({
    success: true,
    dryRun: true,
    wouldCreate: {
      teamId: TEAM_ID,
      title,
      description,
      priority,
      stateId,
      ...(assigneeId && { assigneeId }),
      ...(labelIds.length > 0 && { labelIds }),
    },
    message: "Dry-run mode: ticket not actually created",
  }, null, 2));
  process.exit(0);
}

const mutation = `
  mutation IssueCreate($input: IssueCreateInput!) {
    issueCreate(input: $input) {
      success
      issue {
        id
        identifier
        title
        url
      }
    }
  }
`;

const variables = {
  input: {
    teamId: TEAM_ID,
    title,
    description,
    priority,
    stateId,
    ...(assigneeId && { assigneeId }),
    ...(labelIds.length > 0 && { labelIds }),
  },
};

const response = await fetch("https://api.linear.app/graphql", {
  method: "POST",
  headers: {
    "Content-Type": "application/json",
    Authorization: apiKey,
  },
  body: JSON.stringify({ query: mutation, variables }),
});

const result = await response.json();

if (result.errors) {
  console.error("ERROR:", JSON.stringify(result.errors, null, 2));
  process.exit(1);
}

if (result.data?.issueCreate?.success) {
  const issue = result.data.issueCreate.issue;
  console.log(JSON.stringify({
    success: true,
    id: issue.id,
    identifier: issue.identifier,
    title: issue.title,
    url: issue.url,
  }));
} else {
  console.error("ERROR: Issue creation failed");
  console.error(JSON.stringify(result, null, 2));
  process.exit(1);
}
