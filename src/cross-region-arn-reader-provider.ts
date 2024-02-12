import { Construct } from "constructs";
import {
    CustomResourceProviderBase,
    CustomResourceProviderOptions,
    CustomResourceProviderRuntime,
    Stack,
} from "aws-cdk-lib";
import * as path from "path";

export class CrossRegionArnReaderProvider extends CustomResourceProviderBase {
    /**
     * Returns a stack-level singleton ARN (service token) for the custom resource
     * provider.
     *
     * @param scope Construct scope
     * @param uniqueid A globally unique id that will be used for the stack-level
     * construct.
     * @param props Provider properties which will only be applied when the
     * provider is first created.
     * @returns the service token of the cross region arn reader resource provider, which should be
     * used when defining a `CustomResource`.
     */
    static getOrCreate(
        scope: Construct,
        uniqueid: string,
        props?: CustomResourceProviderOptions,
    ): string {
        return this.getOrCreateProvider(scope, uniqueid, props).serviceToken;
    }

    /**
     * Returns a stack-level singleton for the custom resource provider.
     *
     * @param scope Construct scope
     * @param uniqueid A globally unique id that will be used for the stack-level
     * construct.
     * @param props Provider properties which will only be applied when the
     * provider is first created.
     * @returns the service token of the cross region arn reader resource provider, which should be
     * used when defining a `CustomResource`.
     */
    static getOrCreateProvider(
        scope: Construct,
        uniqueid: string,
        props?: CustomResourceProviderOptions,
    ): CrossRegionArnReaderProvider {
        const id = `${uniqueid}CustomResourceProvider`;
        const stack = Stack.of(scope);
        return (
            (stack.node.tryFindChild(id) as CrossRegionArnReaderProvider) ??
            new CrossRegionArnReaderProvider(stack, id, props)
        );
    }

    constructor(
        scope: Construct,
        id: string,
        props?: CustomResourceProviderOptions,
    ) {
        super(scope, id, {
            ...props,
            codeDirectory: path.join(
                __dirname,
                "function",
                "cross-region-arn-reader",
                "build",
            ),
            runtimeName: CustomResourceProviderRuntime.NODEJS_18_X,
        });
    }
}
