# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.


from concurrent.futures import Executor, Future
from typing import Callable, Any


class SynchronousExecutor(Executor):
    def submit(self, fn: Callable[..., Any], /, *args, **kwargs) -> Future:
        future: Future = Future()
        try:
            result = fn(*args, **kwargs)
            future.set_result(result)
        except Exception as e:
            future.set_exception(e)
        return future

    def shutdown(self, wait=True, *, cancel_futures=False) -> None:
        # No resources to clean up in this simple synchronous executor
        pass
