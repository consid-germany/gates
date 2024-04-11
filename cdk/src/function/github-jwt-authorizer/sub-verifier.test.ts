import { expect, it } from "vitest";
import { JwtPayload } from "aws-jwt-verify/jwt-model";
import { matchesSub } from "./sub-verifier";

it("should match sub from pattern", () => {
    // given
    const payload: JwtPayload = {
        sub: "repo:some-organization/some-repository:ref:refs/heads/main",
    };

    // when
    const result = matchesSub(payload, ["repo:some-organization/some-repository:*"]);

    // then
    expect(result).to.be.true;
});

it("should not match sub from pattern", () => {
    // given
    const payload: JwtPayload = {
        sub: "repo:some-organization/some-other-repository:ref:refs/heads/main",
    };

    // when
    const result = matchesSub(payload, [
        "repo:some-organization/some-repository:*"
    ]);

    // then
    expect(result).to.be.false;
});

it("should match sub from multiple patterns", () => {
    // given
    const payload: JwtPayload = {
        sub: "repo:some-organization/some-repository:ref:refs/heads/main",
    };

    // when
    const result = matchesSub(payload, [
        "repo:some-organization/some-other-repository:*",
        "repo:some-organization/some-repository:*",
    ]);

    // then
    expect(result).to.be.true;
});

it("should not match sub from multiple patterns", () => {
    // given
    const payload: JwtPayload = {
        sub: "repo:some-organization/some-repository:ref:refs/heads/main",
    };

    // when
    const result = matchesSub(payload, [
        "repo:some-organization/some-other-repository:*",
        "repo:some-organization/some-third-repository:*",
    ]);

    // then
    expect(result).to.be.false;
});

it("should not match sub if zero patterns", () => {
    // given
    const payload: JwtPayload = {
        sub: "repo:some-organization/some-repository:ref:refs/heads/main",
    };

    // when
    const result = matchesSub(payload, []);

    // then
    expect(result).to.be.false;
});

it("should not match sub if no sub given", () => {
    // given
    const payload: JwtPayload = {};

    // when
    const result = matchesSub(payload, []);

    // then
    expect(result).to.be.false;
});
