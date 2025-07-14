# Notes

EXTERNAL PACKAGES:

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

Need an entirely new approach for namespace exports since they just don't work.

HIR has redundant fields for namespaces, and neither are really sufficient.

resolver / global scope also has namespaces.

HIR package namespaces, and resolver namespace tree really serve different purposes.

Resolver namespace tree:

- contains all visible namespaces for current package
- easy to look up by name

hir namespace tree/ items

- should be merged
- should have item ids
- should have visibility
- export items should be able to point to them

Open question: do we need the namespace tree structure on the HIR to "optimize"
lookups (even though I think the optimization is kidn of silly)? Or are the
lookups all happening on the resolver's namespace tree anyway so it doesn't matter?

### Trying to figure out namespace representation

    A                B                C

---

    AA             A.AA              -
    AB             A.AB              -
                    BA             B.BA
                    BB             B.BB

### TRANSITIVE NAMESPACE EXPORTS JUST DON'T MAKE SENSE

Why?

OK, so, really, exporting a namespace is just aliasing a namespace.

Example:

```
namespace NamespaceA {
    export NamespaceB;
    export NamespaceB as NamespaceBAlias;
}
```

is just creating two new namespaces, NamespaceA.NamespaceB and NamespaceA.NamespaceB.NamespaceBAlias without affecting the visibility of items declared within NamespaceB.

So consider transitive dependencies now.
A -> B -> C
When C exports an item, Item1, it's just visible to B, which directly depends on C. If B wanted to make Item1
visible to A, it would have to _re-export_ it. Without a re-export, A can't see Item1. In otherwords, Item1 is
public to C but private to B.

What if B re-exported the _namespace_ that contains Item1? Well, as we said above, exporting a namespace doesn't
change the visibility of the items within it. So, even if A "sees" that re-exported namespace from B, it will still
look empty.

In order to include Item1 in the exports, B would have to explicitly export the item. At which point, this is
the declaration you would use in B:

```qsharp
namespace CAlias {
    export C.Item1;
}
```

the namespace export is moot.

So, what shall we do?

- idea 1: warning/error when you reexport a namespace from another package, saying this has no effect
- idea 2: reexporting a namespace reexports all the public items within it, as if we did `export Foo.*` (which isn't a thing today btw)
  - this sounds like a lot of effort to handle a special case, for what exactly?

Going with idea 1 now.

## declaration parts

There's the "name" and there's the "path". distinct concepts
`import A as B` - `B` is the name, `A` is the path
`import A`, no name, `A` is the path
`import A.*` - no name, `A` is the path

## Shadowing/ Collisions

shadowing / collision:

- currently, local declarations shadow opens or glob imports, but direct imports shadow local declarations
- no collisions between globs/opens, but collisions between direct imports
- no collisions between a direct import and a local declaration <-- seems inconsistent with above?

### Design change

- solution --- maybe shadowing should always be allowed in LOCAL scope - to support Jupyter scenarios - just like callables.

Also, this used to work (for some reason) but now it doesn't

import Foo.Bar;
export Bar;

Justification - even though the above "worked" - it was order dependent. global declarations aren't normally order dependent.

## Out of scope, but consider

conditional compilation:

- currently we check dropped_names. Maybe we should require an attribute on the export instead.

## Limitations

### NO GLOB EXPORTS

Preexisting limitation

### No exporting from glob imports (i.e. open namespaces)

I'm not sure if this even works today.

I said this shouldn't be allowed, but reconsidering:
import Foo.\*;
export Bar; // (from Foo)

This is tricky to make work, and Rust also disallows this sort of thing anyway.

`export Bar` can be a:

- same-namespace export, OR
- a namespace direct export

So we can't make assumptions.

WAIT - why shouldn't this be allowed again?
Because, at binding time, we don't know if we're supposed to create a new name in that scope or not.

- if we do create a new name, it can create a conflict with an existing item
- if we don't create a new name, then other references won't be able to resolve to this export
- potential solution: only create a new name if it doesn't exist? the rules can get a bit complicated here.

OK, new approach -

- create a special res for `import Bar`/`export Bar`, doesn't count as a collision

I need to run through some test cases in practice to figure this one out.

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
