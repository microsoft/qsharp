# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.


def fetch_github(owner: str, repo: str, ref: str, path: str) -> str:
    import requests

    path_no_leading_slash = path[1:] if path.startswith("/") else path
    url = f"https://raw.githubusercontent.com/{owner}/{repo}/{ref}/{path_no_leading_slash}"
    return requests.get(url).text
