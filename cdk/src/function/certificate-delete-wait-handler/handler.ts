import { CloudFormationCustomResourceHandler, CloudFormationCustomResourceEvent } from "aws-lambda";
import * as https from "https";
import * as url from "url";

const WAIT_TIME_MS = 60000; // 60 seconds

export const handler: CloudFormationCustomResourceHandler = async (event: CloudFormationCustomResourceEvent) => {
    console.log("Event:", JSON.stringify(event, null, 2));

    try {
        if (event.RequestType === "Delete") {
            console.log(`Waiting ${WAIT_TIME_MS}ms before completing delete...`);
            await new Promise((resolve) => setTimeout(resolve, WAIT_TIME_MS));
            console.log("Wait completed");
        }

        await sendResponse(event, "SUCCESS", event.LogicalResourceId);
    } catch (error) {
        console.error("Error:", error);
        await sendResponse(event, "FAILED", event.LogicalResourceId);
    }
};

async function sendResponse(
    event: CloudFormationCustomResourceEvent,
    status: string,
    physicalResourceId: string,
): Promise<void> {
    const responseBody = JSON.stringify({
        Status: status,
        Reason: `See the details in CloudWatch Log Stream: ${process.env.AWS_LOG_STREAM_NAME}`,
        PhysicalResourceId: physicalResourceId,
        StackId: event.StackId,
        RequestId: event.RequestId,
        LogicalResourceId: event.LogicalResourceId,
        Data: {
            message: "Processed",
        },
    });

    const parsedUrl = url.parse(event.ResponseURL);
    const options = {
        hostname: parsedUrl.hostname,
        port: parsedUrl.port,
        path: parsedUrl.path,
        method: "PUT" as const,
        headers: {
            "content-type": "",
            "content-length": responseBody.length,
        },
    };

    return new Promise((resolve, reject) => {
        const request = https.request(options, (response) => {
            response.on("data", () => {});
            response.on("end", () => {
                console.log("Response sent successfully");
                resolve();
            });
        });

        request.on("error", (error) => {
            console.error("Error sending response:", error);
            reject(error);
        });

        request.write(responseBody);
        request.end();
    });
}

