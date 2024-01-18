# Azure Quantum Resource Estimator

The implementation for the Resource Estimator is broken up into two major components:

- [counts](./src/counts.rs) - performs program execution tracing to capture the logical qubit and gate counts for the given program
- [estimates](./src/estimates.rs) - takes in logical counts and a configuration to produce the set of corresponding physical resource estimates

For more information about the Azure Quantum Resource Estimator, see [the official documentation](https://learn.microsoft.com/en-us/azure/quantum/intro-to-resource-estimation).
