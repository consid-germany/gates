import {afterEach, beforeAll, beforeEach, expect, it, vi} from "vitest";
import * as cdk from "aws-cdk-lib";
import {Template} from "aws-cdk-lib/assertions";
import {execSync} from "child_process";
import {Gates} from "./gates";
import {createTestBuilds} from "../test/assets";

beforeAll(() => {
    execSync("npm run build:function");
    createTestBuilds();
});

beforeEach(() => {
    vi.useFakeTimers();
});

afterEach(() => {
    vi.useRealTimers();
});

it("should match snapshot", async () => {
    // given
    vi.setSystemTime(0);

    const app = new cdk.App();
    const stack = new cdk.Stack(app, "Stack", {
        stackName: "some-stack-name",
        env: { region: "eu-central-1", account: "1234567890" },
    });

    // when
    new Gates(stack, "Gates", {});

    // then
    const template = Template.fromStack(stack);
    expect(template).toMatchSnapshot();
});
