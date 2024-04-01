import * as cdk from "aws-cdk-lib";
import {IConstruct} from "constructs";

export class ApplyDestroyPolicyAspect implements cdk.IAspect {
    public visit(node: IConstruct): void {
        if (node instanceof cdk.CfnResource) {
            node.applyRemovalPolicy(cdk.RemovalPolicy.DESTROY);
        }
    }
}
