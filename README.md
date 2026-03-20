# Lambda calculus parser & interpreter

## Usage

```bash
cargo run -- foo.ln bar.ln bar.ln
```

## Syntax

```text
# Church integers
zero := \f => \x => x
succ := \n => \f => \x => f (n f x)
```
