import * as fs from "fs";
import * as path from "path";

const REPOSITORY_README = path.join(__dirname, "..", "..", "README.md");
const CDK_README = path.join(__dirname, "..", "README.md");

if (fs.existsSync(CDK_README)) {
    fs.rmSync(CDK_README);
}

fs.cpSync(REPOSITORY_README, CDK_README);
