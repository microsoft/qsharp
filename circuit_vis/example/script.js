// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.

if (typeof qviz != 'undefined') {
    document.getElementById('message').remove();

    /* These examples shows how to draw circuits into containers. */
    const entangleDiv = document.getElementById('entangle');
    if (entangleDiv != null) {
        qviz.draw(entangle, entangleDiv, qviz.STYLES['Default']);
    }

    const sampleDiv = document.getElementById('sample');
    if (sampleDiv != null) {
        qviz.draw(sample, sampleDiv, qviz.STYLES['Default']);
    }

    const teleportDiv = document.getElementById('teleport');
    if (teleportDiv != null) {
        qviz.draw(teleport, teleportDiv, qviz.STYLES['Default']);
    }

    const groverDiv = document.getElementById('grover');
    if (groverDiv != null) {
        qviz.draw(grover, groverDiv, qviz.STYLES['Default']);
    }
} else {
    document.getElementById('group').remove();
}
