# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest
import os


@pytest.fixture
def qsharp():
    import qsharp
    import qsharp._fs
    import qsharp._http

    qsharp._fs.read_file = read_file_memfs
    qsharp._fs.list_directory = list_directory_memfs
    qsharp._fs.exists = exists_memfs
    qsharp._fs.join = join_memfs
    qsharp._fs.resolve = resolve_memfs
    qsharp._http.fetch_github = fetch_github_test

    return qsharp


def test_project(qsharp) -> None:
    qsharp.init(project_root="/good")
    result = qsharp.eval("Test.ReturnsFour()")
    assert result == 4


def test_project_compile_error(qsharp) -> None:
    with pytest.raises(Exception) as excinfo:
        qsharp.init(project_root="/compile_error")
    assert str(excinfo.value).startswith("Qsc.TypeCk.TyMismatch")


def test_project_bad_qsharp_json(qsharp) -> None:
    with pytest.raises(Exception) as excinfo:
        qsharp.init(project_root="/bad_qsharp_json")
    assert str(excinfo.value).find("Failed to parse manifest") != -1


def test_project_unreadable_qsharp_json(qsharp) -> None:
    with pytest.raises(Exception) as excinfo:
        qsharp.init(project_root="/unreadable_qsharp_json")
    assert str(excinfo.value).startswith(
        "Error reading /unreadable_qsharp_json/qsharp.json."
    )


def test_project_unreadable_source(qsharp) -> None:
    with pytest.raises(Exception) as excinfo:
        qsharp.init(project_root="/unreadable_source")
    # If this seems like a silly substring to assert on, it's
    # because the error reporting code is inserting a line break
    # between "could not" and "read test.qs"
    assert str(excinfo.value).find("OSError: could not") != -1


def test_project_dependencies(qsharp) -> None:
    qsharp.init(project_root="/with_deps")
    result = qsharp.eval("Test.CallsDependency()")
    assert result == 4


def test_project_circular_dependency_error(qsharp) -> None:
    with pytest.raises(Exception) as excinfo:
        qsharp.init(project_root="/circular")
    assert str(excinfo.value).find("Circular dependency detected between") != -1


def test_github_dependency(qsharp) -> None:
    qsharp.init(project_root="/with_github_dep")
    result = qsharp.eval("Test.CallsDependency()")
    assert result == 12


def test_with_files(qsharp) -> None:
    qsharp.init(project_root="/with_files")
    result = qsharp.eval("Test.ReturnsFour()")
    assert result == 4


def test_relative_project_root(qsharp) -> None:
    chdir_memfs("/")
    qsharp.init(project_root="with_files")
    result = qsharp.eval("Test.ReturnsFour()")
    assert result == 4


def test_relative_project_root_with_dot(qsharp) -> None:
    chdir_memfs("/with_files")
    qsharp.init(project_root=".")
    result = qsharp.eval("Test.ReturnsFour()")
    assert result == 4


# def test_project_root_parent(qsharp) -> None:
#     chdir_memfs("/with_files/src")
#     qsharp.init(project_root="..")
#     result = qsharp.eval("Test.ReturnsFour()")
#     assert result == 4


# def test_project_root_with_dotdot(qsharp) -> None:
#     chdir_memfs("/with_files")
#     qsharp.init(project_root="../with_files")
#     result = qsharp.eval("Test.ReturnsFour()")
#     assert result == 4


memfs = {
    "": {
        "good": {
            "src": {
                "test.qs": "namespace Test { operation ReturnsFour() : Int { 4 } export ReturnsFour; }",
            },
            "qsharp.json": "{}",
        },
        "with_files": {
            "src": {
                "test.qs": "namespace Test { operation ReturnsFour() : Int { 4 } export ReturnsFour; }",
            },
            "qsharp.json": """
                {
                    "files": ["src/test.qs"]
                }""",
        },
        "bad_qsharp_json": {"qsharp.json": "BAD_JSON_CONTENTS"},
        "unreadable_qsharp_json": {
            "qsharp.json": OSError("could not read qsharp.json")
        },
        "unreadable_source": {
            "src": {
                "test.qs": OSError("could not read test.qs"),
            },
            "qsharp.json": "{}",
        },
        "compile_error": {
            "src": {
                "test.qs": "namespace Test { operation ReturnsFour() : Int { 4.0 } }",
            },
            "qsharp.json": "{}",
        },
        "with_deps": {
            "src": {
                "test.qs": "namespace Test { operation CallsDependency() : Int { return Foo.Test.ReturnsFour(); } }",
            },
            "qsharp.json": """
                {
                    "dependencies": {
                        "Foo": {
                            "path": "../good"
                        }
                    }
                }""",
        },
        "circular": {
            "src": {
                "test.qs": "namespace Test {}",
            },
            "qsharp.json": """
                {
                    "dependencies": {
                        "Foo": {
                            "path": "../circular"
                        }
                    }
                }""",
        },
        "with_github_dep": {
            "src": {
                "test.qs": "namespace Test { operation CallsDependency() : Int { return Foo.Test.ReturnsTwelve(); } }",
            },
            "qsharp.json": """
                {
                    "dependencies": {
                        "Foo": {
                            "github" : {
                                "owner" : "test-owner",
                                "repo" : "test-repo",
                                "ref" : "12345"
                            }
                        }
                    }
                }""",
        },
    }
}
memfs_cwd_path = "/"
memfs_cwd = memfs[""]


def fetch_github_test(owner: str, repo: str, ref: str, path: str):
    if (
        owner == "test-owner"
        and repo == "test-repo"
        and ref == "12345"
        and path == "/qsharp.json"
    ):
        return """{ "files" : ["src/test.qs"] }"""
    if (
        owner == "test-owner"
        and repo == "test-repo"
        and ref == "12345"
        and path == "/src/test.qs"
    ):
        return "namespace Test { operation ReturnsTwelve() : Int { 12 } export ReturnsTwelve;}"
    raise Exception(f"Unexpected fetch_github call: {owner}, {repo}, {ref}, {path}")


def chdir_memfs(path):
    global memfs_cwd_path
    global memfs_cwd
    print(f"chdir {path} cwd: {memfs_cwd_path}")
    memfs_cwd_path = resolve_memfs(memfs_cwd_path, path)
    memfs_cwd = memfs[""]
    for part in memfs_cwd_path.split("/"):
        print(f"part: {part}")
        if part == "." or part == "":
            continue
        if part in memfs_cwd:
            memfs_cwd = memfs_cwd[part]
        else:
            raise Exception(f"Directory not found: {path}")
    print(f"chdir {path} -> {memfs_cwd_path}")


def read_file_memfs(path):
    global memfs
    global memfs_cwd
    item = memfs_cwd
    print(f"read_file {path} cwd: {memfs_cwd_path}")
    for part in path.split("/"):
        if part == "." or part == "":
            continue
        if part in item:
            if isinstance(item[part], OSError):
                raise item[part]
            else:
                item = item[part]
        else:
            raise Exception(f"File not found: {path} cwd: {memfs_cwd_path}")

    print(f"read_file {path} -> {item}")
    return (path, item)


def list_directory_memfs(dir_path):
    global memfs
    global memfs_cwd
    item = memfs_cwd
    for part in dir_path.split("/"):
        if part == "." or part == "":
            continue
        if part in item:
            item = item[part]
        else:
            raise Exception(f"Directory not found: {dir_path} cwd: {memfs_cwd_path}")

    contents = list(
        map(
            lambda x: {
                "path": join_memfs(dir_path, x[0]),
                "entry_name": x[0],
                "type": "folder" if isinstance(x[1], dict) else "file",
            },
            item.items(),
        )
    )

    print(f"list_directory({dir_path}) -> {contents}")
    return contents


def exists_memfs(path):
    global memfs
    global memfs_cwd
    item = memfs_cwd
    print(f"exists {path} cwd: {memfs_cwd_path} cur: {item}")
    for part in path.split("/"):
        print(f"part: {part}")
        if part == "." or part == "":
            continue
        if part in item:
            item = item[part]
        else:
            print(f"exists {path} -> False")
            return False

    print(f"exists {path} -> True")
    return True


# The below functions force the use of `/` separators in the unit tests
# so that they function on Windows consistently with other platforms.
def join_memfs(path, *paths):
    if path.endswith("/"):
        path = path[:-1]
    res = "/".join([path, *paths])
    return res


def resolve_memfs(base, path):
    print(f"resolve_memfs({base}, {path})")
    if base.endswith("/"):
        base = base[:-1]
    if path.startswith("/"):
        print("Returning path")
        return path
    absolute = base.startswith("/")
    parts = f"{base}/{path}".split("/")
    new_parts = []
    for part in parts:
        if part == ".":
            continue
        if part == ".." and len(new_parts) > 0 and new_parts[-1] != "..":
            new_parts.pop()
            continue
        new_parts.append(part)

    if len(new_parts) == 0:
        res = "/" if absolute else "."
        print(f"Returning {res}")
        return res

    res = "/".join(new_parts)
    print(f"Returning {res}")
    return res
