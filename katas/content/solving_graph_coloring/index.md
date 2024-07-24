# Solving Graph Coloring Using Grover's Algorithm

@[section]({
    "id": "solving_graph_coloring__overview",
    "title": "Overview"
})

The key part of solving a classical problem using Grover's search algorithm is implementing the quantum oracle for that problem.
In practice, implementing a quantum oracle for a specific problem can be quite challenging.

This kata walks you through implementing the quantum oracles for several kinds of graph coloring problems - 
problems that look for an assignment of labels, commonly referred to as "colors", to vertices or edges of the graph 
in a way that satisfies the given set of constraints.
It also encourages you to experiment with using these oracles to solve graph coloring problems with Grover's search.

**This kata covers the following topics:**

- Implementation of marking oracles for several versions of graph coloring problems
- Using Grover's search algorithm to solve graph coloring problems

**What you should know to start working on this kata:**

- Fundamental quantum concepts
- Controlled gates
- Oracles, in particular marking oracles
- Grover's search algorithm

@[section]({
    "id": "solving_graph_coloring__vertex_coloring",
    "title": "Vertex Coloring Problem"
})

Vertex coloring problem is the simplest form of a graph coloring problem. In it, you look for a coloring of graph vertices which labels each vertex with one of the given colors so that no two vertices of the same color are connected by an edge. In other words, the colors of any pair of vertices that is connected by an edge must be different.

In this lesson, you will implement the marking oracle for the vertex coloring problem, as well as several building blocks you'll need for an end-to-end implementation of Grover's search for this problem.

@[exercise]({
    "id": "solving_graph_coloring__vertex_coloring_classical",
    "title": "Is Vertex Coloring Valid? (Classical)",
    "path": "./vertex_coloring_classical/"
})

@[exercise]({
    "id": "solving_graph_coloring__read_coloring",
    "title": "Read Coloring From a Qubit Array",
    "path": "./read_coloring/"
})

@[exercise]({
    "id": "solving_graph_coloring__color_equality",
    "title": "Are Colors Equal?",
    "path": "./color_equality/"
})

@[exercise]({
    "id": "solving_graph_coloring__vertex_coloring_quantum",
    "title": "Is Vertex Coloring Valid? (Quantum)",
    "path": "./vertex_coloring/"
})


@[section]({
    "id": "solving_graph_coloring__weak_coloring",
    "title": "Weak Coloring Problem"
})

TBD


@[section]({
    "id": "solving_graph_coloring__using_grover",
    "title": "Using Grover's Algorithm to Solve Graph Coloring Problems"
})

In this lesson, you will experiment with using Grover's algorithm to solve graph coloring problems.

Notice that in this case, it's not as easy to know the number of solutions to the problem upfront as it was for the prefix function used in the "Grover's Search Algorithm" kata.
Experiment with choosing the number of iterations at random. How does this affect the success probability?

TBD

@[section]({
    "id": "solving_graph_coloring__conclusion",
    "title": "Conclusion"
})

Congratulations! In this kata you learned to solve graph coloring problems using Grover's search.
