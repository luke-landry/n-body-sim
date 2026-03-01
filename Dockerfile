FROM rust:trixie

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update \
    && apt-get install -y --no-install-recommends \
    # apt packages needed for development
    git sudo mingw-w64 gnuplot nvidia-cuda-toolkit \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/*

# use non-root user for container
ARG USERNAME=dev
ARG USER_UID=1000
ARG USER_GID=1000

RUN groupadd --gid $USER_GID $USERNAME \
    && useradd -u $USER_UID -g $USER_GID -m $USERNAME -s /bin/bash \
    && echo $USERNAME ALL=\(root\) NOPASSWD:ALL > /etc/sudoers.d/$USERNAME \
    && chmod 0440 /etc/sudoers.d/$USERNAME

WORKDIR /home/$USERNAME/n-body-sim

USER $USERNAME

RUN rustup target add x86_64-pc-windows-gnu \
    && rustup component add clippy rustfmt
