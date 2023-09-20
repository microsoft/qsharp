# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

ARG BASE_IMAGE
FROM --platform=arm64 ${BASE_IMAGE}

# install python and pip
RUN apt-get update && \
    DEBIAN_FRONTEND=noninteractive apt-get install \
    python3-minimal python3-pip \
    --no-install-recommends -y && \
    apt-get clean

# Update pip and install required packages
RUN pip install -U pip

ENTRYPOINT ["sh", "-c", "$*", "--"]
