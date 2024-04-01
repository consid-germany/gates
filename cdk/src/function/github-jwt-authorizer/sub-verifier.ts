import {JwtPayload} from "aws-jwt-verify/jwt-model";
import wildcardMatch from 'wildcard-match';

export function matchesSub(payload: JwtPayload, allowedSubPatterns: string[]) {
    if (payload.sub === undefined) {
        return false;
    }
    for (const allowedSubPattern of allowedSubPatterns) {
        const match = wildcardMatch(allowedSubPattern, false);
        if (match(payload.sub)) {
            return true;
        }
    }
    return false;
}
