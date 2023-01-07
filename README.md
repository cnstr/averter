# Canister Averter
This service redirects Canister v1 API requests to the Canister v2 API.<br>
It works by completing the following tasks in order:
* Parsing query parameters and inputs
* Fetching the data using the v2 API routes
* Rearranging the data to match the v1 API responses

### Development
This project utilizes [`Rust`](https://rust-lang.org) and `cargo`.<br>
To build the project, run `cargo build` and to run the project, run `cargo run`.<br>

### Deployment
You shouldn't really be deploying this project on your own (unless you feel like hosting this for some reason).<br>
Deployment utilizes [`task`](https://taskfile.dev) and private Taskfile which contains deployment instructions.<br>
The `task deploy` command will automatically trigger [cnstr/ci](https://github.com/cnstr/ci) which will:
* Build and publish the Docker image to the [tale.me](https://tale.me/docker) registry
* Rewrite the `kubernetes/deployment.yaml` file with the new image tag
* Apply the new deployment to the cluster

> Copyright (c) 2023, Aarnav Tale
