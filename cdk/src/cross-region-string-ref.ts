import * as ssm from "aws-cdk-lib/aws-ssm";
import { Stack } from "aws-cdk-lib";
import { Construct, IConstruct } from "constructs";
import { SSMParameterReader } from "./ssm-parameter-reader";

const CUSTOM_RESOURCE_TYPE = "Custom::CrossRegionStringRef";

export interface CrossRegionStringRefProps<C extends IConstruct> {
    constructInOtherRegion: C;
    value: (construct: C) => string;
}

export default class CrossRegionStringRef<C extends IConstruct> extends Construct {
    private readonly stackOfConstruct: Stack;
    private readonly ssmParameterReader: SSMParameterReader;

    constructor(scope: Construct, id: string, props: CrossRegionStringRefProps<C>) {
        super(scope, id);
        this.stackOfConstruct = Stack.of(props.constructInOtherRegion);

        const parameterName = `/${this.cdkExportsParameterPrefix}/${this.stackOfConstruct.stackName}/${props.constructInOtherRegion.node.path.replace(/[^/\w.-]/g, "-")}/${id}`;

        new ssm.StringParameter(props.constructInOtherRegion, id, {
            parameterName,
            stringValue: props.value(props.constructInOtherRegion),
        });

        this.ssmParameterReader = new SSMParameterReader(this, id, {
            parameterName,
            region: this.stackOfConstruct.region,
            resourceType: CUSTOM_RESOURCE_TYPE,
        });
    }

    get value(): string {
        return this.ssmParameterReader.value;
    }

    private get cdkExportsParameterPrefix() {
        return `cdk/${this.stackOfConstruct.synthesizer.bootstrapQualifier}/exports`;
    }
}
