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

@[exercise]({
    "id": "solving_sat__sat_formula",
    "title": "Evaluate SAT Formula",
    "path": "./sat_formula/"
})


@[section]({
    "id": "solving_sat__exactly_one_3sat",
    "title": "Exactly-1 3-SAT Problem"
})

Exactly-1 3-SAT problem (also known as "one-in-three 3-SAT") is a variant of a general SAT problem.
The structure of the formula that describes an instance of an exactly-1 3-SAT problem is exactly the same as that of the canonical SAT problem, with each clause consisting of exactly three literals.
However, the problem is to find an assignment of the variables that makes each clause have *exactly one* true literal, 
while in a normal SAT problem each clause can have one or more true literal to satisfy the formula.

Formally, the clauses of exactly-1 3-SAT problem can be defined via the use of a ternary operator that is `true` if and only if exactly one of the arguments is `true`. However, this kata uses the same formula notation for these clauses as the canonical SAT problem.

@[exercise]({
    "id": "solving_sat__exactly_one_one",
    "title": "Evaluate \"Exactly 1 One\" Operator",
    "path": "./exactly_one_one/"
})

@[exercise]({
    "id": "solving_sat__exactly_one_one_formula",
    "title": "Evaluate Exactly-1 3-SAT Formula",
    "path": "./exactly_one_one_formula/"
})


@[section]({
    "id": "solving_sat__using_grover",
    "title": "Using Grover's Algorithm to Solve SAT Problems"
})

In this lesson, you will experiment with using Grover's algorithm to solve SAT problems.

Notice that in this case, it's not as easy to know the number of solutions to the problem upfront as it was for the prefix function used in the "Grover's Search Algorithm" kata.
Experiment with choosing the number of iterations at random. How does this affect the success probability?

@[example]({"id": "solving_sat__e2edemo", "codePath": "./examples/SolvingSATWithGroverDemo.qs"})


@[section]({
    "id": "solving_sat__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to solve Boolean satisfiability problems using Grover's search.
