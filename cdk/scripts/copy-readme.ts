import * as fs from "fs";
import * as path from "path";
import { fileURLToPath } from "node:url";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const REPOSITORY_README = path.join(__dirname, "..", "..", "README.md");
const CDK_README = path.join(__dirname, "..", "README.md");

if (fs.existsSync(CDK_README)) {
    fs.rmSync(CDK_README);
}

fs.cpSync(REPOSITORY_README, CDK_README);
