# Wise Analysis of Rust Lines of Code (WarLOC)

There are hundreds of tools that count lines of code in you software project. But have you ever wondered how many of
those lines are inside hundreds of unit tests spread across your Rust modules? And let's be honest, we all what to brag
about our project documentation being bigger than the code itself.

`warloc` can help you with both of those issues and more. Here is what it does you:
* Counts separately lines of codes, documentation comments, regular comments and blank lines.
* Separates those into lines belonging to main code, tests and examples.
* Finds integration tests under `tests` directories.
* Finds unit test code and fixtures under `#[cfg(test)]` and `#[test]`.
* Understands (to a limit) Rust syntax, so is more accurate in its counts then most generic LOC counters.
* Does not count ignored (by `.gitignore`) files.
* Can optionally give you stats file-by-file.

Here is the output for `cargo` repository:
```
File count: 1188
Type         | Code         | Blank        | Doc comments | Comments     | Total       
-------------|--------------|--------------|--------------|--------------|-------------
Main         | 82530        | 9682         | 12625        | 6220         | 111057      
Tests        | 144421       | 20538        | 588          | 10151        | 175698      
Examples     | 169          | 27           | 5            | 19           | 220         
-------------|--------------|--------------|--------------|--------------|-------------
             | 227120       | 30247        | 13218        | 16390        | 286975      
```

# Installation

```shell
cargo install cargo-warloc
```

# Usage

```shell
cargo warloc
```

# Contributing

This project is made for fun, so there are no plans for implementing new features and fixing bugs (unless they feel like
fun). If you like the project and you want it do something it doesn't, submit a PR, I'll gladly merge it (unless it's
buggy or/and unreasonable) and publish the new version.

# License

MIT or Apache 2.0 on your discretion.

# Bragging

Got stats of your project and want to brag about it? Open a new thread in the discussions.
