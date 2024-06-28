# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.

import pytest
import os


@pytest.fixture
def qsharp():
    import qsharp
    import qsharp._fs

    qsharp._fs.read_file = read_file_memfs
    qsharp._fs.list_directory = list_directory_memfs
    qsharp._fs.exists = exists_memfs
    qsharp._fs.join = join_memfs
    qsharp._fs.resolve = resolve_memfs

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
    assert str(excinfo.value).find("OSError: could not read test.qs") != -1


def test_project_dependencies(qsharp) -> None:
    qsharp.init(project_root="/with_deps")
    result = qsharp.eval("Test.CallsDependency()")
    assert result == 4


def test_project_circular_dependency_error(qsharp) -> None:
    with pytest.raises(Exception) as excinfo:
        qsharp.init(project_root="/circular")
    assert str(excinfo.value).find("Circular dependency detected between") != -1


memfs = {
    "": {
        "good": {
            "src": {
                "test.qs": "namespace Test { operation ReturnsFour() : Int { 4 } export ReturnsFour; }",
            },
            "qsharp.json": "{}",
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
    }
}


def read_file_memfs(path):
    global memfs
    item = memfs
    for part in path.split("/"):
        if part in item:
            if isinstance(item[part], OSError):
                raise item[part]
            else:
                item = item[part]
        else:
            raise Exception("File not found: " + path)

    return (path, item)


def list_directory_memfs(dir_path):
    global memfs
    item = memfs
    for part in dir_path.split("/"):
        if part in item:
            item = item[part]
        else:
            raise Exception("Directory not found: " + dir_path)

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

    return contents


def exists_memfs(path):
    global memfs
    parts = path.split("/")
    item = memfs
    for part in parts:
        if part in item:
            item = item[part]
        else:
            return False

    return True


# The below functions force the use of `/` separators in the unit tests
# so that they function on Windows consistently with other platforms.
def join_memfs(path, *paths):
    return "/".join([path, *paths])


def resolve_memfs(base, path):
    parts = f"{base}/{path}".split("/")
    new_parts = []
    for part in parts:
        if part == ".":
            continue
        if part == "..":
            new_parts.pop()
            continue
        new_parts.append(part)
    return "/".join(new_parts)
