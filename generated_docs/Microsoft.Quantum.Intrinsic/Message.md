---
uid Microsoft.Quantum.Intrinsic.Message
title: Message function
ms.date: todo
ms.topic: managed-reference
qsharp.kind: function
qsharp.namespace: Microsoft.Quantum.Intrinsic
qsharp.name: Message
qsharp.summary: Logs a message.
---

# Message function

Namespace: [Microsoft.Quantum.Intrinsic](xref:Microsoft.Quantum.Intrinsic)

Logs a message.
```qsharp
function Message(msg : String) : Unit
```

## Summary
Logs a message.

## Input
### msg
The message to be reported.

## Remarks
The specific behavior of this function is simulator-dependent,
but in most cases the given message will be written to the console.
```
