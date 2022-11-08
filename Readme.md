# Oluś


## Syntax

```
fact n ret ↦ is_zero n base recurse
```

$$
\overbrace{
\overbrace{
    \underbrace{
        \mathsf{fact}
    } _ {\text{name}}
    {\ }
    \underbrace{
        \mathsf{n}
        {\ }
        \mathsf{ret}
    } _ {\text{parameters}}
}^{\text{procedure}}
\overbrace{
    {\ }
    \mathsf{↦}
    {\ }
}^{\text{maplet}}
\overbrace{
    \underbrace{
        \mathsf{is\_zero}
    } _ {\text{closure}}
    {\ }
    \underbrace{
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

## Compilation process

1. [`lexer`] is a [logo] based tokenizer. It recognizes the minimal set of
   tokens the language has and parses nested quoted strings.
2. [`parse`] is a [rowan] based concrete syntax tree. It recognizes indentation
   blocks, lines and matches parenthesis.
