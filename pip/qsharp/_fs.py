# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os


def read_file(path):
    with open(path, mode="r", encoding="utf-8") as f:
        return (path, f.read())


def list_directory(dir_path):
    return list(
        map(
            lambda e: {
                "path": os.path.join(dir_path, e),
                "entry_name": e,
                "type": "file"
                if os.path.isfile(os.path.join(dir_path, e))
                else "folder"
                if os.path.isdir(os.path.join(dir_path, e))
                else "unknown",
            },
            os.listdir(dir_path),
        )
    )


def exists(path):
    return os.path.exists(path)
