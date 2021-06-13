# bliss

> bliss is (or at least will be) an elegant, dynamically typed programming language.

So far, we have a kind of broken tree-walking interpreter. It's a bit buggy, but it works generally. The plan is to turn that into a bytecode VM, but that might take a while to do.

I haven't really put together any resources for learning the language, so you can try to figure it out from the examples for now (if you're brave).

The source code is kind of all over the place so expect a refactor *coming soon to a theater near you*

## Goals of this project

- [x] Have error good and clear parser errors that point you in the right direction (kinda there)
- [ ] Have a powerful standard library, that makes it practical
- [ ] Have decent performance and garbage collection

## Influences

- JavaScript
- Elixir
- Haskell
- [Ink](https://dotink.co)
- [The Interpreter book](https://interpreterbook.com/)

## Known Bugs/issues

- [ ] For some reason non-tail recursion doesn't work. See [this example](examples/fib.bliss)
  - [ ] Recursion in general is kinda broken tbh
- [ ] There is no garbage collection so the stack will overflow if you do too much recursion
- [ ] The parser can only output one error at a time

I plan to fix these eventually, so don't worry too much

## Directory Structure

- `/lib`: The library that powers the language. You'll find the lexer, parser, semantic analyzer, and evaluator here
- `/src`: The REPL and file reader
- `/grammar`: An experimental VS Code extension for syntax highlighting
- `/examples`: A couple examples of code (used for testing)
