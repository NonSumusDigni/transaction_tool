# transaction_tool
----------------

A command-line tool that reads in a CSV of transactions and produces to stdout a CSV a set of client account balances and state.

(For a coding exercise with a prompt)

## Running

Run the tool on an input CSV using cargo run: `cargo run -- test-data/test.csv`

Run the tests with `cargo test`

## Notes

### Completeness

All expected cases are handled, and every invalid transaction shape I could think of is accounted for.

### Correctness

Functionality of the CLI is verified using the `test.csv` found in the test-data directory. This CSV also exercises a handful of code paths, including some invalid transactions.

Most of the functionality verification takes place in the tests found at the bottom of the processor.rs file. There is not 100% coverage of every error/invalid case, but the important business logic is exercised.

### Safety and robustness

All unrecoverable errors are propagated up to the top level main function, which writes out the error message and exits the process with status code 1.

Per the specification, various forms of invalid records are ignored. In this implementation they are ignored silently, in a real system we'd want to collect them and surface them to the user in some useful fashion.

### Efficiency

The CSV is processed one row at a time, and not all of the data derived from these rows is held in memory at once. Each row is deserialized into a `Transaction` struct and passed into the processor. The processor is ultimately a pure function which takes in a `State` and a `Transaction`, and returns a `State`. The `State` is what contains and owns the relevant data which is maintened - client account info (balances and locked status) and some relevant transactions (only those which can be referenced by other transactions are kept*).

What this means is that the memory usage is unbounded and grows in proportion to the size of the input dataset, albeit with some savings of free'd Transaction allocations (the disputes/resolves/chargebacks). In a real-world system handling data of this sort, we'd want to persist the State information outside of memory (probably a SQL store and a cache), both for the persistence's sake itself and to avoid using all the memory. Of course, at that point we wouldn't be dealing with a simple command line tool.

<sub>\* There was some uncertainty about whether a withdrawal could be disputed or charged back. Ultimately, based on the descriptions in the prompt and in thinking through the real-world reference of such a system, I decided that only deposits could be targetted with disputes and chargebacks. However, I am still maintaining reference to the withdrawals in the state in order to detect re-used transaction IDs.</sub>

### Maintainability

The code is organized into three files - processor.rs contains the core functionality, main.rs handles all of the CLI logic and the input/output serialization/deserialization, and types.rs contains the struct and enum definitions which are referenced by both of the aforementioned files.

There is some code repetition in processor.rs, between the 5 different transaction type handlers. Some thought was given to extracting out some of the similar logic for re-use, but the variations between the handlers are such that it would have made the code less readable, and so less maintainable.

Implementing `process_transaction`, the entry point to the processing code, as a pure function lends to its maintainability, as it is simple to reason about and test.