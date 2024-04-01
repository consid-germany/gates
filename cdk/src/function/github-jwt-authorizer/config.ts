export interface GitHubAuthConfig {
    issuer: string;
    jwksUri: string;
    audience: string;
    allowedSubPatterns: string[];
}

const ALLOWED_SUB_PATTERNS = "ALLOWED_SUB_PATTERNS";

export function getConfig(): GitHubAuthConfig {
    return {
        issuer: "https://token.actions.githubusercontent.com",
        jwksUri: "https://token.actions.githubusercontent.com/.well-known/jwks",
        audience: "consid-germany/gates",
        allowedSubPatterns: getAllowedSubPatterns(),
    };
}

function getAllowedSubPatterns(): string[] {
    const allowedSubPatterns = JSON.parse(getEnvVariable(ALLOWED_SUB_PATTERNS));
    if (!isArrayOfStrings(allowedSubPatterns)) {
        throw new Error("could not parse allowedSubPatterns");
    }
    return allowedSubPatterns;
}

function getEnvVariable(envVarName: string): string {
    const envVar = process.env[envVarName];

    if (!envVar) {
        throw new Error(`${envVarName} environment variable is not set`);
    }

    return envVar;
}

function isArrayOfStrings(value: unknown): value is string[] {
    return Array.isArray(value) && value.every(item => typeof item === "string");
}
