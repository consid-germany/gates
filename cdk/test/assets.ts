import * as path from "path";
import * as fs from "fs";

export function createTestBuilds() {
    const TEST_API_DIR = path.join(__dirname, "assets", "api");
    const TEST_UI_DIR = path.join(__dirname, "assets", "ui");

    const BUILD_DIR = path.join(__dirname, "..", "build");

    if (fs.existsSync(path.join(BUILD_DIR, "api"))) {
        fs.rmSync(path.join(BUILD_DIR, "api"), {recursive: true});
    }

    if (fs.existsSync(path.join(BUILD_DIR, "ui"))) {
        fs.rmSync(path.join(BUILD_DIR, "ui"), {recursive: true});
    }

    fs.cpSync(TEST_API_DIR, path.join(BUILD_DIR, "api"), {recursive: true});
    fs.cpSync(TEST_UI_DIR, path.join(BUILD_DIR, "ui"), {recursive: true});
}
