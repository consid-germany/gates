import { beforeEach, expect, it, vi } from "vitest";
import * as core from "@actions/core";
import { run } from "./main";

global.fetch = vi.fn();

beforeEach(() => {
    vi.resetAllMocks();
    vi.mock("@actions/core", () => ({
        getInput: vi.fn(),
        getBooleanInput: vi.fn(),
        getIDToken: vi.fn(),
        setFailed: vi.fn(),
        notice: vi.fn(),
    }));
});

it("should not fail and set notice if gate is open", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch).mockResolvedValue({
        status: 200,
        json: () =>
            Promise.resolve({
                state: "open",
            }),
    } as Response);

    // when
    await run();

    // then
    expect(core.notice).toHaveBeenCalledWith(
        "Gate some-test-group/some-test-service/some-test-environment is open.",
    );
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
});

it("should fail and set failed if gate is closed", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch).mockResolvedValue({
        status: 200,
        json: () =>
            Promise.resolve({
                state: "closed",
            }),
    } as Response);

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith(
        "Gate some-test-group/some-test-service/some-test-environment is closed.",
    );
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
});

it("should fail and set failed if gate could not be found", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch).mockResolvedValue({
        status: 204,
    } as Response);

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith(
        "Gate some-test-group/some-test-service/some-test-environment could not be found.",
    );
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
});

it("should fail and create gate if gate could not be found", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });
    vi.mocked(core.getBooleanInput).mockImplementation((input) => {
        switch (input) {
            case "create_gate_if_not_exists":
                return true;
        }
        return false;
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch)
        .mockResolvedValueOnce({
            status: 204,
        } as Response)
        .mockResolvedValueOnce({
            status: 200,
        } as Response);

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith(
        "Gate some-test-group/some-test-service/some-test-environment could not be found.",
    );
    expect(core.notice).toHaveBeenCalledWith(
        "Created gate some-test-group/some-test-service/some-test-environment.",
    );
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
    expect(fetch).toHaveBeenCalledWith("https://github.some.gates.deployment.com/api/gates", {
        method: "POST",
        body: JSON.stringify({
            group: "some-test-group",
            service: "some-test-service",
            environment: "some-test-environment",
        }),
        headers: {
            Accept: "application/json",
            Authorization: "Bearer some-github-jwt",
            "User-Agent": "consid-germany/gates",
        },
    });
});

it("should fail but not create gate if gate could not be found", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });
    vi.mocked(core.getBooleanInput).mockImplementation((input) => {
        switch (input) {
            case "create_gate_if_not_exists":
                return false;
        }
        return false;
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch).mockResolvedValue({
        status: 204,
    } as Response);

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith(
        "Gate some-test-group/some-test-service/some-test-environment could not be found.",
    );
    expect(core.notice).not.toHaveBeenCalledWith(
        "Created gate some-test-group/some-test-service/some-test-environment.",
    );
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
});

it("should fail for errors when creating gate if gate could not be found", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });
    vi.mocked(core.getBooleanInput).mockImplementation((input) => {
        switch (input) {
            case "create_gate_if_not_exists":
                return true;
        }
        return false;
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch)
        .mockResolvedValueOnce({
            status: 204,
        } as Response)
        .mockResolvedValueOnce({
            status: 500,
        } as Response);

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith(
        "Gate some-test-group/some-test-service/some-test-environment could not be found.",
    );
    expect(core.setFailed).toHaveBeenCalledWith(
        "Error: Request to create gate some-test-group/some-test-service/some-test-environment failed: 500 undefined",
    );
    expect(core.notice).not.toHaveBeenCalledWith(
        "Created gate some-test-group/some-test-service/some-test-environment.",
    );
    expect(fetch).toHaveBeenCalledTimes(2);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
    expect(fetch).toHaveBeenCalledWith("https://github.some.gates.deployment.com/api/gates", {
        method: "POST",
        body: JSON.stringify({
            group: "some-test-group",
            service: "some-test-service",
            environment: "some-test-environment",
        }),
        headers: {
            Accept: "application/json",
            Authorization: "Bearer some-github-jwt",
            "User-Agent": "consid-germany/gates",
        },
    });
});

it("should fail and set failed if internal error occurred", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch).mockResolvedValue({
        status: 500,
        statusText: "Some internal error",
    } as Response);

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith(
        "Request to check gate some-test-group/some-test-service/some-test-environment failed: 500 Some internal error",
    );
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
});

it("should fail and set failed if getting input fails", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation(() => {
        throw new Error("Some error");
    });

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith("Error: Some error");
    expect(fetch).toHaveBeenCalledTimes(0);
});

it("should fail and set failed if getting id token fails", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });

    vi.mocked(core.getIDToken).mockRejectedValue(new Error("Some error"));

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith("Error: Some error");
    expect(fetch).toHaveBeenCalledTimes(0);
});

it("should fail and set failed if fetching gate state fails", async () => {
    // given
    vi.mocked(core.getInput).mockImplementation((input) => {
        switch (input) {
            case "gitHubApiBaseUrl":
                return "https://github.some.gates.deployment.com/api";
            case "group":
                return "some-test-group";
            case "service":
                return "some-test-service";
            case "environment":
                return "some-test-environment";
        }
        return "";
    });

    vi.mocked(core.getIDToken).mockResolvedValue("some-github-jwt");

    vi.mocked(fetch).mockRejectedValue(new Error("Some error"));

    // when
    await run();

    // then
    expect(core.setFailed).toHaveBeenCalledWith("Error: Some error");
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(fetch).toHaveBeenCalledWith(
        "https://github.some.gates.deployment.com/api/gates/some-test-group/some-test-service/some-test-environment/state",
        {
            method: "GET",
            headers: {
                Accept: "application/json",
                Authorization: "Bearer some-github-jwt",
                "User-Agent": "consid-germany/gates",
            },
        },
    );
});
