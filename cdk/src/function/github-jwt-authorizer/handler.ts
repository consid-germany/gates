import {JwtRsaVerifier} from "aws-jwt-verify";
import {getConfig} from "./config";
import {matchesSub} from "./sub-verifier";
import {APIGatewayRequestSimpleAuthorizerHandlerV2} from "aws-lambda/trigger/api-gateway-authorizer";

const BEARER = "Bearer ";

const CONFIG = getConfig();

const JWT_RSA_VERIFIER = JwtRsaVerifier.create({
    issuer: CONFIG.issuer,
    audience: CONFIG.audience,
    jwksUri: CONFIG.jwksUri,
    customJwtCheck: async ({ payload }) => {
        if (!matchesSub(payload, CONFIG.allowedSubPatterns)) {
            throw new Error("sub not allowed");
        }
    },
});

export const handler: APIGatewayRequestSimpleAuthorizerHandlerV2 = async (event) => {
    const authorization = event.headers?.authorization;

    if (!authorization) {
        return {
            isAuthorized: false,
        };
    }

    const token = authorization.replace(BEARER, "");

    try {
        await JWT_RSA_VERIFIER.verify(token);
    } catch {
        return {
            isAuthorized: false,
        };
    }

    return {
        isAuthorized: true,
    };
};
