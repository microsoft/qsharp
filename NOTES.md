# Notes

## Inventory of actual changes

- Disallow exporting primitive types like Unit, Qubit, Int... and remove them from the Microsoft.Quantum.Core legacy api
  - These exports were not actually working, and there was no error message about them either, so this isn't a take-back
- HIR export items were not being generated in some cases, so cross-package exports didn't work (e.g. the FixedPoint stuff) . Previously we were only generating
  export items for aliased exports (and reexports from other packages).
- HIR Export items are now of type Res, not ItemId - so that they can have an Err value
  - No critical reason - just more align with how we lower other references, and technically other items could refer
    to an export item, so the item IDs should be valid in the HIR.
- Invalid patterns such as `import A.* as B` and `export C.*` are now detected in the parser, to reduce the amount validation
  that the resolver ends up having to do. We can now assume that we won't encounter these conditions in the resolver.
- Enumerating HIR globals - instead of "Term" I'm calling it a "Callable" because that's what it is
  - Double check Python behavior - were UDTs supposed to show up under qsharp.code?
- Cross-namespace exports are not allowed anymore.
  - The individual items in the namespace would still be invisible to any consumers, making the namespace reexport effectively useless.
- Fixed resolution of namespace reexports under `Main` from dependent packages e.g.:

  namespace A {
  function B() : Unit {}
  export B;
  }
  namespace Main {
  export A as AAlias;
  }

- Fixed: This type of export, used in the Stdlib legacy api, was not working:

  import Foo.\*;
  export Bar;

- Item IDs in the HIR got shifted forward across the board, because now we assign all import and export items IDs during the binding step.
- Moved a lot of tests from out of incremental/tests.rs and into multiple_packages.rs to consolidate related tests together.
  - incremental test helper was just... weird. I deleted it. It was taking 2 vector arguments that were never used as vectors.
- I'm calling it a "wildcard" instead of a "glob", since I believe that to be the more accurate term, and that's what our public docs call it anyway (https://learn.microsoft.com/en-us/azure/quantum/how-to-work-with-qsharp-projects?tabs=tabid-qsharp%2Ctabid-qsharp-run#using-the-import-statement)
- Parent namespace exports are explicitly banned now, since they were never fully supported anyway. We cannot generate proper HIR for this since parent namespaces are not assigned item IDs.

```
namespace Parent.Foo {
  function Bar() : Unit {}
  export Bar;
}

namespace Baz {
  export Parent as ParentAlias;
}
```

- Removed the unnecessary aliases from our libraries (fixed point, rotations) that were needed as workarounds
- In namespace scopes, duplicate imports/exports are now always errors. This is consistent with how other item declarations behave, such as `operation Foo`, `struct Bar` etc.
  It's also an error when the name of an import collides with an callable/udt declaration
  - Previously, it seems that this was allowed so that rerunnig jupyter cells would work. This is a fair expectation, however, jupyter cells don't run in namespace contexts.
    This problem was already solved for function/UDT declarations previously by allowing duplicate names in LOCAL scopes. We do the same for imports now.

## Fresh strategy

pass 1: bind all declarations (not imports or exports)

pass 2:

- in a loop:
  - bind opens and glob imports
  - resolve all imports (and exports, which are considered imports++)
  - whenever an import is resolved, bind the name
  - keep going in a loop until no new imports have been resolved in a pass
  - (retry namespace imports since they may eventually resolve to item imports?)

pass 3:

- bind opens and glob imports
- resolve all other declarations

## older strategy

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

is just creating two new namespaces, NamespaceA.NamespaceB and NamespaceA.NamespaceBAlias without affecting the visibility of items declared within NamespaceB.

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

### PARENT NAMESPACES

Parent namespaces don't have item ids. and it turns out it's just a pain to make them have item ids.

Today, exports of parent namespaces don't work because we can't create export items for them in the HIR.
By "don't work" I mean: they're not accessible from outside the package. They're accessible in the same package.

So I'm making a call to ban them for now instead of trying to fix this. I tried various approaches
to make it work and they were all terrible in some way.

Workaround: instead of `export ParentNamespace` write:

```
namespace MyNamespace.ParentNamespace {
    export ParentNamespace.ChildNamespace1;
    export ParentNamespace.ChildNamespace1;
}
```

c'est la vie.

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

## The name resolution behavior table of doom:

| **Caller**                                              | Python interop | Code in same package <br>/<br> Entry expr in same package | Code in notebook cell <br>/<br> Entry expr from interpreter commands <br> /<br> Some code lens actions | Code in dependent package | Code in dependent package (stdlib special case) |
| :------------------------------------------------------ | -------------- | --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------ | ------------------------- | ----------------------------------------------- |
| **Item visibility**                                     | all items      | all items                                                 | all items                                                                                              | only exports              | only exports                                    |
| **`Main` namespace treatment**                          | `Main` exists  | `Main` exists                                             | `Main` exists                                                                                          | `Main` is collapsed       | N/A, no `Main` in stdlib                        |
| **Package alias?**                                      | No             | No                                                        | No                                                                                                     | Yes                       | No                                              |
| **Calling code lives in same or separate HIR package?** | N/A, no code   | Same                                                      | Separate                                                                                               | Separate                  | Separate                                        |

## Random todos

- rename glob -> wildcard
- Variational Quantum Algorithms.ipynb is showing squiggles and I don't understand why
  2025-07-29 12:23:35.916 [error] Unexpected error trying to read file Unable to read file - '\\c:\src\qsharp\samples\chemistry\spsa\qsharp.json' (Unknown (FileSystemError): UNC host 'c:' access is not allowed. Please update the 'security.allowedUNCHosts' setting if you want to allow this host.)
