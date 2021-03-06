<p align="center">
    <img src="logo.svg" width="50%" />
</p>

<p align="center">
    <img alt="GitHub issues" src="https://img.shields.io/github/issues/trimorphdev/cherry?style=for-the-badge">
    <img alt="CircleCI" src="https://img.shields.io/circleci/build/github/trimorphdev/cherry/main?style=for-the-badge">
    <img alt="GitHub" src="https://img.shields.io/github/license/trimorphdev/cherry?color=blue&style=for-the-badge">
</p>

## Table of Contents
- [About](#about)
- [Project Checklist](#project-checklist)
- [Examples](#examples)

## About
Cherry is a weakly statically typed, type safe, memory managed programming language inspired by both TypeScript and Rust.  Cherry's goals are:

- To be type and memory safe
- To eliminate as much implicity as reasonably possible
- To be extremely fast, like Rust, C and C++

Now, here's what Cherry's goals *aren't*:

- Cherry is not intended to be "the easiest programming language to learn."  It will never be the easiest programming language to learn.  In fact, it's one of Cherry's side goals to be a decent challenge to use, but not to the point of being a pain.  For example, Rust is a decently difficult programming language to learn, but it provides amazing error messages and documentation to help point you in the right direction.  It's like a puzzle, not meant to be agonizing to use, but enough challenge to not be completely boring.

## Project Checklist
> **note:** This section will be removed when the project is past the alpha and beta stages.
- [x] The lexer.
- [x] The command line interface.
    - [x] Diagnostic emitter, using `codespan-reporting`.
        - [x] Add multiple available themes.
        - [x] Allow themes to be changed on command line interface
- [ ] The parser.
    - [ ] Basic literals.
    - [ ] Operator precedence.
    - [ ] Type annotations.
    - [ ] `let` and `const` statements.
    - [ ] Control statements, like `for`, `while`, `if` and `match`.
    - [ ] Function definitions.
    - [ ] Data structures; `enum`, `struct` and `type`.
    - [ ] Import statements.
- [ ] The code generator.

## Examples
> **note:** It is not guaranteed that these examples will work when Cherry is finished, as its syntax may change.

### Hello, world!
```cherry
export function main() {
    println("Hello, world!");
}
```