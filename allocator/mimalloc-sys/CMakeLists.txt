# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

cmake_minimum_required(VERSION 3.10.0)


project(allocator_external)
include(ExternalProject)

ExternalProject_Add(mimalloc
  GIT_REPOSITORY    https://github.com/microsoft/mimalloc.git
  GIT_TAG           $ENV{ALLOCATOR_MIMALLOC_TAG}
  GIT_SHALLOW       TRUE
  GIT_PROGRESS      TRUE
  CONFIGURE_COMMAND ""
  BUILD_COMMAND ""
  INSTALL_COMMAND ""
  TEST_COMMAND ""
  USES_TERMINAL_DOWNLOAD  TRUE
)
