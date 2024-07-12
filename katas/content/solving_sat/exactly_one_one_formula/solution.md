This exercise is similar to the task of evaluating a SAT formula from the first lesson: it consists of a conjunction (AND) of results of multiple clause evaluations. This time, though, each clause has to be evaluated using the "Exactly One 1" policy you've implemented in the previous exercise instead of the `Oracle_Or` you used in the first lesson.

With this replacement, the code that evaluates a single clause is very similar to that from the exercise "Evaluate One Clause",
and the code that evaluates the whole formula - to that from the exercise "Evaluate SAT Formula".

@[solution]({
    "id": "solving_sat__exactly_one_one_formula_solution",
    "codePath": "./Solution.qs"
})
