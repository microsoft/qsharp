# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

cmake_minimum_required(VERSION 3.10.0)

# Copy the command line arguments to a cache variable so that they can be used
# with the inner project. This is needed because the inner project
# needs to use the same configuration and the command line arguments are not
# passed to the inner project.
# This also manifests itself in apple silicon systems where the inner project
# defaults to x86_64 even if the outer project is arm64.
# This needs to be done before the project command is called.
if (NOT ALLOCATOR_CMAKE_CL_ARGS)
  # get all cache variables
  get_cmake_property(vars CACHE_VARIABLES)
  foreach(var ${vars})
    get_property(currentHelpString CACHE "${var}" PROPERTY HELPSTRING)
    # only add the cache variable to the list if it is set on the command line
    if("${currentHelpString}" MATCHES "No help, variable specified on the command line." OR "${currentHelpString}" STREQUAL "")
      message ("${var} = [${${var}}]  --  ${currentHelpString}")
      list(APPEND ALLOCATOR_CMAKE_CL_ARGS "-D${var}=${${var}}")
    endif()
  endforeach()
  # cache the command line arguments
  set(ALLOCATOR_CMAKE_CL_ARGS ${ALLOCATOR_CMAKE_CL_ARGS} CACHE STRING "comment")
endif ()

project(allocator_external)
include(ExternalProject)

ExternalProject_Add(mimalloc
  GIT_REPOSITORY    https://github.com/microsoft/mimalloc.git
  GIT_TAG           $ENV{ALLOCATOR_MIMALLOC_TAG}
  GIT_SHALLOW       TRUE
  GIT_PROGRESS      TRUE
  CMAKE_ARGS        ${ALLOCATOR_CMAKE_CL_ARGS}
  BUILD_COMMAND     ninja
  USES_TERMINAL_DOWNLOAD  TRUE
  USES_TERMINAL_CONFIGURE TRUE
  USES_TERMINAL_BUILD     TRUE
  USES_TERMINAL_INSTALL   TRUE
)
