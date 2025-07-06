add items from external packages to global scope

BINDING:
for each scope (namespace, callable, block)

- add the opens
- glob imports are just opens
- opens with an alias are the same as imported namespaces with an alias
- bind all the names for declared items
- for direct imports and exports, bind the names
- exports are just treated as imports here
- collisions are detected between item declarations and imports (or maybe just for global scope- see notes below)
- special case for `export A` without alias or qualifier: don't bind the names

RESOLVING:

- all imports must be resolved first, down to the original item decl, so we can differentiate tys, terms, and namespaces
- resolve all the paths

EXPORT BINDING:
(perhaps this could be the done in the RESOLVING step)

- for same namespace exports, just make item public
- add export items for all other exports
- collisions/shadowing should have applied in the binding step

## namespace exports

## declaration parts

There's the "name" and there's the "path". distinct concepts
`import A as B` - `B` is the name, `A` is the path
`import A`, no name, `A` is the path
`import A.*` - no name, `A` is the path

## Shadowing/ Collisions:

shadowing / collision:

- currently, local declarations shadow opens or glob imports, but direct imports shadow local declarations
- no collisions between globs/opens, but collisions between direct imports
- no collisions between a direct import and a local declaration <-- seems inconsistent with above?

### Design change

- solution --- maybe shadowing should always be allowed in LOCAL scope - to support Jupyter scenarios - just like callables.

## Out of scope, but consider

conditional compilation:

- currently we check dropped_names. Maybe we should require an attribute on the export instead.

## Limitations

### NO GLOB EXPORTS!

Preexisting limitation

### No exporting from glob imports (i.e. open namespaces)

I'm not sure if this even works today.

Also not allowed (out of inconvenience):
import Foo.*;
export Bar; // (from Foo)
// This is tricky to make work, and Rust also disallows this sort of thing anyway
// `export Bar` is *ALWAYS\* a special-case for same-namespace exports.

WAIT - why shouldn't this be allowed again?
Because, at binding time, we don't know if we're supposed to create a new name in that scope or not.

- if we do create a new name, it can create a conflict with an existing item
- if we don't create a new name, then other references won't be able to resolve to this export
- potential solution: only create a new name if it doesn't exist? the rules can get a bit complicated here.

## Current Bugs (I think)

### Inconsistent behavior between glob imports and opens

namespace Foo {
import Qux.\*; // this gives a spurious error
open Qux; // this works
}

namespace Foo.Qux {
function C() : Unit {}
}

### Namespace reexports from other packages

A

B.namespaceC.Foo(); // this doesn't work

B

export namespaceC;

C

namespaceC { function Foo() : Unit {} s }
