# Copyright (c) Microsoft Corporation.
# Licensed under the MIT License.


import pytest

from qsharp import TargetProfile
from utils import (
    assert_strings_equal_ignore_line_endings,
    compile_qsharp,
    get_input_files,
    get_output_ll_file,
    get_output_out_file,
    QIR_RUNNER_AVAILABLE,
    read_file,
    save_qir_to_temp_file_and_execute,
    SKIP_REASON,
)


@pytest.mark.parametrize("file_path", get_input_files(TargetProfile.Adaptive_RI))
@pytest.mark.skipif(not QIR_RUNNER_AVAILABLE, reason=SKIP_REASON)
def test_adaptive_ri_qir(file_path: str) -> None:
    source = read_file(file_path, TargetProfile.Adaptive_RI)
    ll_file_path = get_output_ll_file(file_path, TargetProfile.Adaptive_RI)
    expected_qir = read_file(ll_file_path, TargetProfile.Adaptive_RI)
    actual_qir = compile_qsharp(
        source,
    )
    assert actual_qir == expected_qir


@pytest.mark.parametrize("file_path", get_input_files(TargetProfile.Adaptive_RI))
@pytest.mark.skipif(not QIR_RUNNER_AVAILABLE, reason=SKIP_REASON)
def test_adaptive_ri_output(file_path: str) -> None:
    source = read_file(file_path, TargetProfile.Adaptive_RI)
    qir = compile_qsharp(source, TargetProfile.Adaptive_RI)
    output_file_path = get_output_out_file(file_path, TargetProfile.Adaptive_RI)
    expected_output = read_file(output_file_path, TargetProfile.Adaptive_RI)
    actual_output = save_qir_to_temp_file_and_execute(qir)
    assert_strings_equal_ignore_line_endings(actual_output, expected_output)
