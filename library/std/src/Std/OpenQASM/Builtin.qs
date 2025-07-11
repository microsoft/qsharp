// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

/// This file defines the bultin functions used in the OpenQASM runtime.
/// It is an internal implementation detail for OpenQASM compilation
/// and is not intended for use outside of this context.

/// OpenQASM only supports up to seven dimensions,
/// therefore, we only need these seven `sizeof` functions.
export sizeof_1, sizeof_2, sizeof_3, sizeof_4, sizeof_5, sizeof_6, sizeof_7;

/// Function to handle 1-dimensional arrays passed to `sizeof` in OpenQASM.
///
/// It takes two arguments, an array `array` and a zero-based dimension `dim`.
/// It returns the length of the requested dimension.
/// Fails if the requested dimension is greater than the number of dimensions
/// in the array.
function sizeof_1<'T>(array : 'T[], dim : Int) : Int {
    if dim == 0 {
        Length(array)
    } else {
        fail $"sizeof error: requested dimension {dim} but the array has 1 dimension";
    }
}

/// Function to handle 2-dimensional arrays passed to `sizeof` in OpenQASM.
///
/// It takes two arguments, an array `array` and a zero-based dimension `dim`.
/// It returns the length of the requested dimension.
/// Fails if the requested dimension is greater than the number of dimensions
/// in the array.
function sizeof_2<'T>(array : 'T[][], dim : Int) : Int {
    if dim == 0 {
        Length(array)
    } elif dim < 2 {
        sizeof_1(array[0], dim - 1)
    } else {
        fail $"sizeof error: requested dimension {dim} but the array has 2 dimensions";
    }
}

/// Function to handle 3-dimensional arrays passed to `sizeof` in OpenQASM.
///
/// It takes two arguments, an array `array` and a zero-based dimension `dim`.
/// It returns the length of the requested dimension.
/// Fails if the requested dimension is greater than the number of dimensions
/// in the array.
function sizeof_3<'T>(array : 'T[][][], dim : Int) : Int {
    if dim == 0 {
        Length(array)
    } elif dim < 3 {
        sizeof_2(array[0], dim - 1)
    } else {
        fail $"sizeof error: requested dimension {dim} but the array has 3 dimensions";
    }
}

/// Function to handle 4-dimensional arrays passed to `sizeof` in OpenQASM.
///
/// It takes two arguments, an array `array` and a zero-based dimension `dim`.
/// It returns the length of the requested dimension.
/// Fails if the requested dimension is greater than the number of dimensions
/// in the array.
function sizeof_4<'T>(array : 'T[][][][], dim : Int) : Int {
    if dim == 0 {
        Length(array)
    } elif dim < 4 {
        sizeof_3(array[0], dim - 1)
    } else {
        fail $"sizeof error: requested dimension {dim} but the array has 4 dimensions";
    }
}

/// Function to handle 5-dimensional arrays passed to `sizeof` in OpenQASM.
///
/// It takes two arguments, an array `array` and a zero-based dimension `dim`.
/// It returns the length of the requested dimension.
/// Fails if the requested dimension is greater than the number of dimensions
/// in the array.
function sizeof_5<'T>(array : 'T[][][][][], dim : Int) : Int {
    if dim == 0 {
        Length(array)
    } elif dim < 5 {
        sizeof_4(array[0], dim - 1)
    } else {
        fail $"sizeof error: requested dimension {dim} but the array has 5 dimensions";
    }
}

/// Function to handle 6-dimensional arrays passed to `sizeof` in OpenQASM.
///
/// It takes two arguments, an array `array` and a zero-based dimension `dim`.
/// It returns the length of the requested dimension.
/// Fails if the requested dimension is greater than the number of dimensions
/// in the array.
function sizeof_6<'T>(array : 'T[][][][][][], dim : Int) : Int {
    if dim == 0 {
        Length(array)
    } elif dim < 6 {
        sizeof_5(array[0], dim - 1)
    } else {
        fail $"sizeof error: requested dimension {dim} but the array has 6 dimensions";
    }
}

/// Function to handle 7-dimensional arrays passed to `sizeof` in OpenQASM.
///
/// It takes two arguments, an array `array` and a zero-based dimension `dim`.
/// It returns the length of the requested dimension.
/// Fails if the requested dimension is greater than the number of dimensions
/// in the array.
function sizeof_7<'T>(array : 'T[][][][][][][], dim : Int) : Int {
    if dim == 0 {
        Length(array)
    } elif dim < 7 {
        sizeof_6(array[0], dim - 1)
    } else {
        fail $"sizeof error: requested dimension {dim} but the array has 7 dimensions";
    }
}
