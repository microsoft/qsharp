// Copyright (c) Microsoft Corporation.
// Licensed under the MIT License.

import {writeFileSync}  from "node:fs";
import {dirname, join} from "node:path";
import {fileURLToPath} from "node:url";
import {inspect} from "node:util";

import {katasMetadata} from "../katas/content/dist/metadata.js"

console.log("CESARZC: BUILD");

const thisDir = dirname(fileURLToPath(import.meta.url));
const katasMetadataJs = join(thisDir, "dist", "katas-metadata.js");

for(const m of katasMetadata.modules)
{
    console.log(`KATAS MODULE: ${m.title}`);
}

console.log(katasMetadataJs);
var obj = {a: 1, b: 2};
writeFileSync(katasMetadataJs, 'var obj = ' + inspect(obj) , 'utf-8');