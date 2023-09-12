# Q# extension for JupyterLab

## Prerequisites

You will need to install `jupyterlab` to build and develop the extension.

```bash
pip install jupyterlab==4.0.0
```

## A note about `yarn`

The `jlpm` command used below is an alias for `yarn` which comes bundled with Jupyter.

This folder is not part of the `npm` workspace that is defined at the root of this repo.
This is intentional, since Jupyter extension tooling depends on `jlpm`, which expects a
`yarn`-style workspace and a `yarn.lock` file.

## Building

To build and install the extension in development mode, from the `jupyterlab` directory run:

```bash
pip install -e .
# The below command creates a symlink from JupyterLab's
# extensions directory to the current source directory.
jupyter labextension develop . --overwrite
```

To start JupyterLab and use the extension, run:

```bash
jupyter lab
```

To rebuild after making source changes, run:

```bash
jlpm build
```

Then refresh the browser.

## Uninstalling

Remove the `pip` package by running:

```bash
pip uninstall qsharp-jupyterlab
```

You will also need to remove the symlink created by `jupyter labextension develop`
command. To find its location, you can run `jupyter labextension list` to figure out where the `labextensions` folder is located. Then you can remove the symlink named `qsharp-jupyterlab` within that folder.

## Releasing

The extension can be published to `PyPI` and `npm` manually or using the [Jupyter Releaser](https://github.com/jupyter-server/jupyter_releaser).
