# gates

[![ci](https://github.com/consid-germany/gates/actions/workflows/ci.yaml/badge.svg)](https://github.com/consid-germany/gates/actions/workflows/ci.yaml)
[![release](https://github.com/consid-germany/gates/actions/workflows/release.yaml/badge.svg)](https://github.com/consid-germany/gates/actions/workflows/release.yaml)

**gates** is an open-source tool that assists software development 
teams in managing the deployment of software artifacts across multiple pipelines and environments.
The tool provides toggles (gates) which can be in the state `open` or `closed`
to control whether a deployment or pipeline should proceed or not.

**gates** comprises the following three components:

1. **api**: HTTP API to create, list and update gates. The API is implemented as a serverless application with Rust, AWS Lambda and the AWS API Gateway. The gates are stored in a AWS DynamoDB table.
2. **ui**: Web frontend providing a user interface to view and toggle gates as well as to add comments to gates (useful to indicate why a gate is in a certain state).
3. **action**: GitHub Action which can be used within GitHub Action workflows to check the state of a gate and stop the pipeline if the gate is in `closed` state.

## Demo

You can check out a demo of the **gates** ui here: https://gates.consid.tech.

*Note that this is a demo deployment of the **gates** application which replaces comment messages with sanitized strings and denies to
create or delete gates.*

## Quick Start - GitHub Action

To use **gates** within your GitHub deployment pipeline you can simply use the ***consid-germany/gates*** action as shown in the block below.
The parameters explained:
```gitHubApiBaseUrl``` points to your version of gates and ```group```, ```service```, ```environment``` are mandatory for identification of the gate, which state is checked. 
If the response is ```open``` the action will continue.

```yaml
jobs:
  example:
    permissions:
      id-token: write
    runs-on: ubuntu-latest
    steps:
      - uses: consid-germany/gates@v1.10.0
        with:
          gitHubApiBaseUrl: https://github.gates.consid.tech/api
          group: some-group
          service: some-service
          environment: test
```

## Quick Start - Rust

There are predefined [run configurations](api/.run) for JetBrains IDEs like RustRover. Otherwise use the commands below.

### Run unit tests

```bash
cd api
cargo test --all-features
```

### Local Testing against API

The Rust backend is provided via [OpenAPI](openapi.yaml).

Requirements:
- Docker
- Cargo Lambda

#### Installing Cargo Lambda (on Mac)

```bash
brew tap cargo-lambda/cargo-lambda
brew install cargo-lambda
```

#### Running the backend locally

1. start docker
2. run dynamodb-local:

   ```bash
   docker run --rm -d --name dynamodb-local -p 8000:8000 amazon/dynamodb-local
   ```
3. run cargo lambda:

   ```bash
   cd api
   cargo lambda watch --features local
   ```

4. backend is now accessible via: http://localhost:9000/api/

## Quick Start - Deployment

The simplest and fastest way to get up and running with **gates** 
from scratch is to deploy the stack in your AWS account with the provided AWS CDK construct. Follow the instructions below.

### 1) Prerequisites

In order to deploy the **gates** application, you will need to meet the following requirements:

- AWS Account with [CDK Bootstrapping](https://docs.aws.amazon.com/cdk/v2/guide/bootstrapping.html) (`us-east-1` region is required to be bootstrapped)
- [AWS CLI](https://docs.aws.amazon.com/cli/latest/userguide/getting-started-install.html) with [configured credentials](https://docs.aws.amazon.com/cli/latest/reference/configure/) for your AWS Account
- [Node.js & npm](https://nodejs.org/en/download)

### 2) Create a new CDK app project

Create a new AWS CDK project using the AWS CDK CLI `cdk init` command:

```bash
npx cdk@latest init app --language typescript
```

### 3) Install the `@consid-germany/gates` package

Install the `@consid-germany/gates` npm package which contains the AWS CDK construct.

```bash
npm install -D @consid-germany/gates
```

### 4) Create stack and `Gates` construct

Inside your CDK app or stack (see `bin` or `lib` directory), import the `Gates` construct from the `@consid-germany/gates` 
package and create it.
The `Gates` construct needs to be created within a stack that has an environment (`env`) configuration providing the `region` and `account` of 
the target AWS account where the application should be deployed.

```ts
import * as cdk from "aws-cdk-lib";
import { Gates } from "@consid-germany/gates";

const app = new cdk.App();

const stack = new cdk.Stack(app, 'Stack', {
    env: {
        region: "eu-central-1", // replace with the region where you want to deploy the stack
        account: "1234567890",  // replace with your AWS account id
    }
});

new Gates(stack, "Gates", {
    gitHubApi: {
        allowedSubPatterns: [
            "repo:consid-germany/gates:*"   // replace with your repositories
        ]
    },
});
```

### 5) Deploy the app

TODO

```bash
npx cdk@latest deploy
```

## Advanced Configuration

TODO

### Use a custom domain

TODO

### Restrict access by IP CIDRs

TODO
