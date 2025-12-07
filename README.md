# n-body-sim

## Build

### Using the VSCode Dev Containers extension
1. Launch the development container by following by selecting the `Reopen in Container` option in the Dev Containers notification after opening the project, or use the `>Dev Containers: Rebuild and Reopen in Container` command in the command palette.
2. Run `cargo build`

### Manually
1. Build the development image with `docker build -t n-body-sim .`
2. Run the development container with `docker run -dit -v $(pwd):/home/dev/n-body-sim --name n-body-sim n-body-sim`
3. Enter the container with `docker exec -it n-body-sim bash`
4. Run `cargo build`
