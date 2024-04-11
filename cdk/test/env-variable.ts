export function getEnvVariable(envVarName: string): string {
    const envVar: string | undefined = process.env[envVarName];

    if (!envVar) {
        throw new Error(`${envVarName} environment variable is not set`);
    }

    return envVar;
}
