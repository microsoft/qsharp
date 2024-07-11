# Solving SAT Problem Using Grover's Algorithm

@[section]({
    "id": "solving_sat__overview",
    "title": "Overview"
})

The key part of solving a classical problem using Grover's search algorithm is implementing the quantum oracle for that problem.
In practice, implementing a quantum oracle for a specific problem can be quite challenging.

This kata walks you through implementing the quantum oracles for Boolean satisfiability problems, abbreviated as SAT problems,
and using these oracles to solve these problems with Grover's search.

**This kata covers the following topics:**

- Implementation of marking oracles for Boolean logic operators and Boolean expressions
- Implementation of marking oracles for several versions of SAT problems

**What you should know to start working on this kata:**

- Fundamental quantum concepts
- Controlled gates
- Oracles, in particular marking oracles
- Grover's search algorithm

@[section]({
    "id": "solving_sat__cnf",
    "title": "Canonical Satisfiability Problem"
})

Boolean satisfiability problem is the problem of determining whether there exists an assignment of Boolean variables $x_j$
for which the given Boolean formula evaluates to true.

The canonical representation for SAT problems is based on the conjunctive normal form of Boolean formulas.
To define this form, we'll need several definitions:

- The Boolean variables $x_0, ..., x_{n-1}$ are the inputs to the formula.
- A _literal_ is either a variable $x_j$ (a positive literal) or the negation of a variable $\neg x_j$ (called a negative literal).
- A _clause_ is a disjunction (an OR operation) of one or several literals, for example, $x_0 \vee \neg x_1$.
  Generally, 
  $$clause(x) = \bigvee_k y_{k},\text{ where }y_{k} =\text{ either }x_j\text{ or }\neg x_j\text{ for some }j \in \{0, \dots, n-1\}$$

- Finally, the _conjunctive normal form_ of a formula is a conjunction (an AND operation) of several clauses:
  $$f(x) = \bigwedge_i \big(\bigvee_k y_{ik} \big),\text{ where }y_{ik} =\text{ either }x_j\text{ or }\neg x_j\text{ for some }j \in \{0, \dots, n-1\}$$

In this lesson, you will learn to implement marking oracles for SAT problems given as their conjunctive normal form descriptions.

@[exercise]({
    "id": "solving_sat__and",
    "title": "Evaluate AND Operator",
    "path": "./and/"
})

@[exercise]({
    "id": "solving_sat__or",
    "title": "Evaluate OR Operator",
    "path": "./or/"
})

@[exercise]({
    "id": "solving_sat__sat_clause",
    "title": "Evaluate One Clause",
    "path": "./sat_clause/"
})


@[section]({
    "id": "solving_sat__exactly_one_3sat",
    "title": "Exactly-1 3-SAT Problem"
})

@[section]({
    "id": "solving_sat__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to solve Boolean satisfiability problems using Grover's search.
