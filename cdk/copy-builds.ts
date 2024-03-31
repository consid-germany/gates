import * as fs from "fs";
import * as path from "path";

const BUILD_DIR = path.join(__dirname, "build");
const _DIR = path.join(__dirname, "build");
const BUILD_DIR = path.join(__dirname, "build");

if (fs.existsSync(BUILD_DIR)) {
    fs.rmSync(BUILD_DIR, { recursive: true });
}

fs.mkdirSync(BUILD_DIR);

fs.cpSync();
