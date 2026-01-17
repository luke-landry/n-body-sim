# n-body-sim

## Build

### Using the VS Code Dev Containers extension
1. In VS Code, install the "Dev Containers" extension and then open this project
2. Launch the development container by following by selecting the `Reopen in Container` option in the Dev Containers notification, or use the `>Dev Containers: Rebuild and Reopen in Container` command in the command palette.
3. Run `cargo build` in the VS Code terminal

### Manually with Docker installed
1. Build the development image with `docker build -t n-body-sim .`
2. Run the development container with `docker run -dit -v $(pwd):/home/dev/n-body-sim --name n-body-sim n-body-sim`
3. Enter the container with `docker exec -it n-body-sim bash`
4. In the container, run `cargo build`
