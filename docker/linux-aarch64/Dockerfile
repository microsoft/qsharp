# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

ARG BASE_IMAGE
FROM --platform=linux/arm64/v8 ${BASE_IMAGE}

# install python and pip
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install \
    python3-minimal python3-pip python3-venv \
    --no-install-recommends -y && \
    apt-get clean

# We don't update pip here as we need to update it
# inside the virtual environment. Otherwise, we get two versions
# of pip installed, and the one outside the virtual environment
# causes problems.

ENTRYPOINT ["sh", "-c", "$*", "--"]
