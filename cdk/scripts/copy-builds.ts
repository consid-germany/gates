import * as fs from "fs";
import * as path from "path";

const API_BUILD_DIR = path.join(__dirname, "..", "..", "api", "target", "lambda", "gates-api");
const UI_BUILD_DIR = path.join(__dirname, "..", "..", "ui", "build");

const BUILD_DIR = path.join(__dirname, "build");

if (fs.existsSync(BUILD_DIR)) {
    fs.rmSync(BUILD_DIR, { recursive: true });
}

fs.mkdirSync(BUILD_DIR);

fs.cpSync(API_BUILD_DIR, path.join(BUILD_DIR, "api"), {recursive: true});
fs.cpSync(UI_BUILD_DIR, path.join(BUILD_DIR, "ui"), {recursive: true});
