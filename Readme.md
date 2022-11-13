# Oluś

**Example.** The factorial function:

```
factorial n ret:
    is_zero n (:ret 1) (:sub n 1 (m:))
    ret (mul n (factorial m))
```

## Basic syntax

Without syntax sugar and just basic syntax, the above example looks like

```
factorial n ret: is_zero n base decrement
base: ret 1
decrement: sub n 1 recurse
recurse m: factorial m induct
induct r: mul n r ret
```

In this form, the order of lines does not matter and the program is just a collection of *definitions* that look like

$$
\overbrace{
\overbrace{
    \underbrace{\vphantom{\_}
        \mathsf{factorial}
    } _ {\text{name}}
    \underbrace{\vphantom{\_}
        \mathsf{n}
        {\ }
        \mathsf{ret}
    } _ {\text{parameters}}
    \!\!\!
}^{\text{procedure}}
\overbrace{\vphantom{f}
    {\ }
    \mathsf{:}
    {\ }
}^{\text{maplet}}
\overbrace{
    \underbrace{
        \mathsf{is\_zero}
    } _ {\text{closure}}
    {\ }
    \underbrace{\vphantom{\_}
        \mathsf{n}
        {\ }
        \mathsf{base}
        {\ }
        \mathsf{recurse}
    } _ {\text{arguments}}
}^{\text{call}}
}^{\text{definition}}
$$

We prefer the generic term *procedure* over function because the language has no intrinsic concept of returning a value.

### Literals

```
“asdasd”
```

Strings are delimited by curved opening `“` and closing `”` quotes. Nesting is supported,


### Functions

A common class of *procedures* are *functions*. These look like

```
func param_1 param_2 … param_n ret ↦ …
ret return_value
```

That is, the last argument is a closure taking in a single value called the *return value*. By convention this parameter is named `ret` (though `k` is common in CPS literature).

Similarly a closure from a function is called *functional*.

## Syntax sugar

### Scopes

### Continued definitions

### Inline definitions

### Inline functionals

Given a *functional* closure `func`, we introduce the notation

```
my_proc a b c ↦ proc a (func a b) c
```

which desugars into:

```
my_proc a b c ↦ func a b (r: proc a r c)
```

An apparent special case is when the notation is in the closure position

```
my_proc a b c ↦ (func a b) c
```

but this desugars similar to above to

```
my_proc a b c ↦ func a b (r: r c)
```

## Compilation process

1. [`lexer`] is a [logo] based tokenizer. It recognizes the minimal set of
   tokens the language has and parses nested quoted strings.
2. [`parse`] is a [rowan] based concrete syntax tree. It recognizes indentation
   blocks, lines and matches parenthesis.


### Closures

```
factorial n ret ↦ is_zero n base decrement
base ↦ ret 1
decrement ↦ sub n 1 recurse
recurse m ↦ factorial m induct
induct r ↦ mul n r ret
```

```
factorial:   []
base:        [ret]
decrement:   [n, recurse]
recurse:     [induct]
induct:      [n, ret]
```

## To do

* Name resolution.
* Basic interpreter.
* Determine closures.
* Control flow analysis.
  http://janmidtgaard.dk/papers/Midtgaard-CSur-final.pdf
* Full type checking.
