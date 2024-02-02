# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import os
from typing import Dict, List, Tuple


def read_file(path: str) -> Tuple[str, str]:
    with open(path, mode="r", encoding="utf-8-sig") as f:
        return (path, f.read())


def list_directory(dir_path: str) -> List[Dict[str, str]]:
    def map_dir(e: str) -> Dict[str, str]:
        return {
            "path": os.path.join(dir_path, e),
            "entry_name": e,
            "type": "file"
            if os.path.isfile(os.path.join(dir_path, e))
            else "folder"
            if os.path.isdir(os.path.join(dir_path, e))
            else "unknown",
        }

    return list(map(map_dir, os.listdir(dir_path)))


def exists(path) -> bool:
    return os.path.exists(path)


def join(path: str, *paths) -> str:
    return os.path.join(path, *paths)
