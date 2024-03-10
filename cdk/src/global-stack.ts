import { Stack, StackProps } from "aws-cdk-lib";
import { IConstruct } from "constructs";

const GLOBAL_REGION = "us-east-1";
const GLOBAL_STACK_ID = "Global";

export default class GlobalStackProvider {
    /**
     * Provides a global stack (stack in the region `us-east-1`) for the given construct.
     * If a global stack has already been created for the scope, the existing global stack is returned.
     * @param scope The construct to create the global stack for.
     * @param props Stack properties.
     */
    static getOrCreate(scope: IConstruct, props?: StackProps): Stack {
        const stack = Stack.of(scope);
        let globalStack = scope.node.tryFindChild(GLOBAL_STACK_ID) as Stack;
        if (!globalStack) {
            globalStack = new Stack(scope, GLOBAL_STACK_ID, {
                ...props,
                env: {
                    region: GLOBAL_REGION,
                    account: stack.account,
                },
            });
            stack.addDependency(globalStack);
        }
        return globalStack;
    }
}
