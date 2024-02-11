import { CdkCustomResourceHandler } from "aws-lambda/trigger/cdk-custom-resource";
import { GetParameterCommand, SSMClient } from "@aws-sdk/client-ssm";

export const handler: CdkCustomResourceHandler = async (event) => {
    const props = event.ResourceProperties;

    if (event.RequestType === "Create" || event.RequestType === "Update") {
        const ssm = new SSMClient({ region: props.Region });

        const ssmParameter = await ssm.send(
            new GetParameterCommand({
                Name: props.ParameterName,
            }),
        );

        return {
            Data: {
                Arn: ssmParameter?.Parameter?.Value,
            },
        };
    }

    return {};
};
