# See here for image contents: https://github.com/microsoft/vscode-dev-containers/tree/v0.187.0/containers/typescript-node/.devcontainer/base.Dockerfile

# [Choice] Node.js version: 16, 18, 20, 22
# bookworm has ARM support
ARG VARIANT="22-bookworm"
FROM mcr.microsoft.com/vscode/devcontainers/typescript-node:${VARIANT} AS base

# add verbosity
ENV DEBIAN_FRONTEND=noninteractive

# https://blog.hyperknot.com/p/corepacks-packagemanager-field
# https://github.com/nodejs/corepack/issues/485
ENV COREPACK_ENABLE_AUTO_PIN=0

# https://consoledonottrack.com/
ENV DO_NOT_TRACK=1
ARG CLAUDE_CODE_VERSION=latest

# --------------------↓-FROM CLAUDE CODE DEVCONTAINER-↓--------------------
# this entire block is copied from the claude code devcontainer
# https://github.com/anthropics/claude-code/blob/main/.devcontainer/Dockerfile#L3-L82

ARG TZ
ENV TZ="$TZ"

ARG CLAUDE_CODE_VERSION=latest

# Install basic development tools and iptables/ipset
RUN apt-get update && apt-get install -y --no-install-recommends \
  less \
  git \
  procps \
  sudo \
  fzf \
  zsh \
  man-db \
  unzip \
  gnupg2 \
  gh \
  iptables \
  ipset \
  iproute2 \
  dnsutils \
  aggregate \
  jq \
  nano \
  vim \
  && apt-get clean && rm -rf /var/lib/apt/lists/*

# Ensure default node user has access to /usr/local/share
RUN mkdir -p /usr/local/share/npm-global && \
  chown -R node:node /usr/local/share

ARG USERNAME=node

# Persist bash history.
RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  && mkdir /commandhistory \
  && touch /commandhistory/.bash_history \
  && chown -R $USERNAME /commandhistory

# Set `DEVCONTAINER` environment variable to help with orientation
ENV DEVCONTAINER=true

# Create workspace and config directories and set permissions
RUN mkdir -p /workspace /home/node/.claude && \
  chown -R node:node /workspace /home/node/.claude

WORKDIR /workspace

ARG GIT_DELTA_VERSION=0.18.2
RUN ARCH=$(dpkg --print-architecture) && \
  wget "https://github.com/dandavison/delta/releases/download/${GIT_DELTA_VERSION}/git-delta_${GIT_DELTA_VERSION}_${ARCH}.deb" && \
  sudo dpkg -i "git-delta_${GIT_DELTA_VERSION}_${ARCH}.deb" && \
  rm "git-delta_${GIT_DELTA_VERSION}_${ARCH}.deb"

# Set up non-root user
USER node

# Install global packages
ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
ENV PATH=$PATH:/usr/local/share/npm-global/bin

# Set the default shell to zsh rather than sh
ENV SHELL=/bin/zsh

# Set the default editor and visual
ENV EDITOR=nano
ENV VISUAL=nano

# Default powerline10k theme
ARG ZSH_IN_DOCKER_VERSION=1.2.0
RUN sh -c "$(wget -O- https://github.com/deluan/zsh-in-docker/releases/download/v${ZSH_IN_DOCKER_VERSION}/zsh-in-docker.sh)" -- \
  -p git \
  -p fzf \
  -a "source /usr/share/doc/fzf/examples/key-bindings.zsh" \
  -a "source /usr/share/doc/fzf/examples/completion.zsh" \
  -a "export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  -x

# Install Claude
RUN npm install -g @anthropic-ai/claude-code@${CLAUDE_CODE_VERSION}

# --------------------↑-FROM CLAUDE CODE DEVCONTAINER-↑--------------------


# --↓-ALSO FROM CLAUDE CODE DEVCONTAINER BUT NOT WORKING YET-↓-------------
# TODO - get this to work by importing the `init-firewall.sh` script and customizing the
# domains at https://github.com/anthropics/claude-code/blob/main/.devcontainer/init-firewall.sh#L68-L76
# this will likely take some trial and error to get it to work with the additional customizations

# Copy and set up firewall script
# COPY init-firewall.sh /usr/local/bin/
# USER root
# RUN chmod +x /usr/local/bin/init-firewall.sh && \
#   echo "node ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/node-firewall && \
#   chmod 0440 /etc/sudoers.d/node-firewall
# USER node
# --↑-ALSO FROM CLAUDE CODE DEVCONTAINER BUT NOT WORKING YET-↑-------------


# --------------------↓-CUSTOMIZATIONS FOR NODE USER-↓--------------------


# Install mise and run mise install in the same layer
RUN curl https://mise.run/zsh | sh && \
    /home/node/.local/bin/mise trust && \
    /home/node/.local/bin/mise install

# Install uv package manager (required for Spec-Kit)
# https://docs.astral.sh/uv/
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# RUN npm i -g rulesync

# Aliases
# RUN echo "alias claude=\"claude --dangerously-skip-permissions\"" >> "/home/node/.zshrc"

