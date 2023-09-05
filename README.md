Recursive Calculator (recalc)
=============================

### 🚧 Work in Progress! 🚧

The Recursive Calculator (recalc) is a calculator that's also a programming language!
It supports all the basic operations: addition (`+`), subtraction and negation (`-`), multiplication (`*`), division (`/`) and exponentiation (`^`),
and even though it doesn't support more advanced operations like logarithms or trigonometric functions, you can define them yourself!

Getting Started
---------------

To get started, start recalc, type a mathematical expression, for example: `1 + 1` and press enter. You'll see the output: `2`.
Or enter: `2 * (2 + 3)^2` and you'll see the output: `50`.<br>
You can also define constants and functions. Try entering:

    pi = 3.14
    f(x) = x^2
    f(pi)

and you'll see the output: `9.8596` (pi squared).<br>

If you have some functions and constants you use very often, you can save their definitions in a file.
Then, every time you run recalc, enter `:load <filename>.recalc` to load them (replace `<filename>` with the name of the file).
Or, if you don't want to write your own file, you can use the predefined [lib.recalc](lib.recalc),
which contains many useful functions and constants.

You can also add comments to your constants, functions and calculations to make them easier to read.
Comments start with two or more asterisks (*) and end with the same number of asterisks, for example:

    ** This is a comment. **
    pi = 3.14
    **** 3.14 is a pretty
    bad approximation of π. ****
    f(x) = pi - x  *** Subtract x from pi. ***

recalc always stores the results of calculations precisely and never rounds them.
For example, if you enter something like `4 / 6`, recalc will store the exact result: _two thirds_,
and not an approximation of the result like `0.6666666666666667`.
When displaying the result on the screen, recalc _will_ round it to make it easier to read,
but this behavior can be disabled with the command `:format fraction`.<br>
The fact that recalc stores numbers precisely also means that it can't compute things like `2 ^ 0.5` (the square root of 2),
which is an [irrational](https://en.wikipedia.org/wiki/Irrational_number) number, and has an infinite number of digits.
In addition, recalc can't handle [imaginary](https://en.wikipedia.org/wiki/Imaginary_number) and [complex](https://en.wikipedia.org/wiki/Complex_number) numbers.

To exit recalc, enter `:quit`.

Undefined Values
----------------

The real power of recalc comes from the way it handles undefined values.
An Undefined value is the result of any expression that recalc can't compute, for example:

- Anything divided by zero, e.g. `1 / 0`.
- zero raised to a negative power, e.g. `0 ^ -1`.
- Irrational roots, e.g. `2 ^ 0.5`.
- Imaginary roots, e.g. `(-1) ^ 0.5`.
- A constant or a function defined as an infinite loop, e.g. `f(3)`, where `f(x)` is defined as `f(x) = f(x) + 1`.<br>

If the output of an expression entered into recalc is undefined, that's [undefined behavior](https://en.wikipedia.org/wiki/Undefined_behavior) -
in that case, recalc may crash, hang forever, or give the wrong result.
However, if an expression contains some parts that are undefined, it doesn't necessarily mean that the whole expression is undefined.
For example, even though recalc can't compute `2 ^ 0.5` (the square root of 2) because it's irrational, it can still compute `2^0.5 * 0`,
because it knows that anything multiplied by zero is zero.

Specifically, the following expressions with undefined values are defined:

- Anything times zero: `x * 0 = 0 * x = 0`, even if `x` is undefined.
- Anything raised to the 0th: `x ^ 0 = 1`, even if `x` is undefined or zero.
- 1 raised to any power: `1 ^ x = 1`, even if `x` is undefined.
- A user-defined function that doesn't depend on its parameter, for example, if `f(x)` is defined as `f(x) = 3`, then `f(x)` is 3 even if `x` is undefined.

*Implementation details:* due to the [halting problem](https://en.wikipedia.org/wiki/Halting_problem),
it's not always possible to determine if a certain value is defined or undefined.
recalc manages to follow all the above rules without always knowing if every value is defined or undefined

Some Useful Functions
---------------------

If you write a file with commonly used functions, as suggested above, you can add these:

    ** Absolute value: **
    abs(x) = (x ^ 2) ^ 0.5

    ** Absolute difference: **
    Δ(x, y) = abs(x - y)

    ***
    Test for nonzero:
    Returns 0 if x = 0, or 1 otherwise.
    (Note that when x = 0, the expression "1 / x" is undefined,
    but the whole expression: "1 / x * x" is still defined.)
    ***
    bool(x) = 1 / x * x

    ***
    Signum:
    Returns -1 if x < 0.
    Returns 0 if x = 0.
    Returns 1 if x > 0.
    (Note the behavior of undefined values here.)
    ***
    sgn(x) = 1 / x * abs(x)

    ***
    Test for zero:
    Returns 1 if x = 0, or 0 if x > 0.
    Undefined if x < 0.
    ***
    not(x) = 0 ^ x

A Simple Recursive Function
---------------------------

In this example, we define the [factorial](https://en.wikipedia.org/wiki/Factorial) function in recalc to show you the power of undefined values.<br>
The factorial function (denoted `n!` or `fact(n)`) is often defined by the following rules:

    1. If n = 0, n! = 1.
    2. If n > 0, n! = n * (n - 1)!
    3. If n is is negative or not an integer, n! is undefined.

In recalc, we can define it more concisely:

    fact(n) = 0^n + n*fact(n - 1)

*How this works:*<br>
The expression `0^n` evaluates to 1 if `n` is 0, or 0 otherwise. So it evaluates to the factorial of `n` only if `n` is 0.<br>
The expression `n*fact(n - 1)` evaluates to 0 if `n` is 0 (even though in that case `fact(n - 1)` is undefined!),
and evaluates to the factorial of `n` otherwise. So it evaluates to the factorial of `n` only if `n` is greater than 0.<br>
We have defined `fact(n)` as the sum of these two values,
so it will evaluate to the factorial of `n` both when `n` is 0 and when it is greater than 0; in other cases, it will be undefined.

Approximating Irrational Numbers
--------------------------------

As you already saw, recalc doesn't support computation with irrational numbers (like `2 ^ 0.5`).
We can still find rational approximations of such numbers, for example, we can use the [Taylor series](https://en.wikipedia.org/wiki/Taylor_series):

    exp(x) = x^0 / 0! + x^1 / 1! + x^2 / 2! + x^3 / 3! + ...

to find an approximation of the [exponential function](https://en.wikipedia.org/wiki/Exponential_function) (denoted `exp(x)` or `e^x`).<br>
First, we'll need a function that returns 0 if its argument is negative, or 1 otherwise:

    checkSign(n) = 0 ^ (abs(n) - n)

Then, we'll write a function that returns the first `n + 1` terms of the above Taylor series:

    expN(x, n) = checkSign(n) * (x^n / fact(n) + expN(x, n - 1))

This function works by the same principle as our factorial function.
The greater the `n` we pass to it, the closer its result will be to the true, irrational value of `exp(x)`.

Finally, we can write our approximate `exp(x)`:

    exp(x) = expN(x, 50)

We arbitrarily chose to pass `n = 50` to `expN`, which will give us relatively precise results for most purposes.

We can also find an approximation of [e](https://en.wikipedia.org/wiki/E_(mathematical_constant)):

    e = exp(1)

(By the way, our `expN(x, n)` is defined in a very inefficient way. Try to see how you can improve it!)

Using the recalc Program
------------------------

When starting recalc, you can either run it in interactive mode, or load recalc code from a file.
To run recalc interactively, simply run the recalc executable. To run code from a file,
either pass the file to the recalc executable as a command line argument, or run recalc interactively and then enter `:load <filename>`.<br>
There are also special commands you can use in interactive mode:

- To quit, enter `:quit` or `:q`.
- To delete a constant or a function, enter `:delete <name>` or `:d <name>`.
- To load a file, enter `:load <filename>` or `:l <filename>`.
- To change the format in which recalc displays its output, enter `:format <format>` or `:f <format>`.
  `<format>` should be one of:
  * `scientific` (default), e.g. `1.333333333333333`.
  * `fraction`, e.g. `4/3`.
  * `mixed`, e.g. `1 + 1/3`.
- To get help, enter `:help` or `:h`.

Notes and Edge Cases:
---------------------

To make this description of recalc a bit more formal, here is a list of special rules and edge cases recalc follows:

- Lines entered into recalc may be: computations (e.g. `1 + 1`), definitions (e.g. `x = 1` or `f(x) = x`),
  special commands (e.g. `:quit`) or empty lines (including lines that contain only whitespace characters and comments), which are ignored.
  Special commands may not contain comments.
- Names of constants, functions and parameters may contain alphabetic characters, digits, underscores (`_`) and apostrophes (`'`).
  They must begin with an alphabetic character or an underscore.
- You may use square brackets `[]` or curly brackets `{}` interchangeably with round ones `()`.
  Each closing bracket must match the opening bracket.
- Comments start with two or more asterisks (`**`) and end with the same number of asterisks.
  All characters within comments, including the asterisks themselves, are ignored.
  A comment may not be immediately preceded or followed by a multiplication (`*`) operator, without a space between them
  (recalc currently does allow this due to a bug).
- You can define functions with multiple parameters, for example: `f(x, y) = x + y`.
  However, you can't return multiple values from a function.
- You can use the unary plus operator, e.g. `x` and `+x` mean the same thing.
- Operator precedence:
  * Operators have the usual precedence: `^`, then `*` and `/`, then `+` and `-`.
  * The `*`, `/`, `+` and `-` are evaluated from left to right.
  * The `^` operator is evaluated from right to left, e.g. `2 ^ 3 ^ 4` is the same as `2 ^ (3 ^ 4)`, not `(2 ^ 3) ^ 4`.
  * Negation (`-`) operators to the left of the `^` operator are evaluated after it, for example `-2^3` is the same as `-(2^3)`, not `(-2) ^ 3`.
  * Negation (`-`) operators to the right of the `^` operator are evaluated before it, e.g. `2 ^ -3` is the same as `2 ^ (-3)`.
- recalc computes powers (e.g `x ^ y`) by the following algorithm:
  1. If `x = 1` or `y = 0`, the result is 1.
  2. If `x = 0`:
      * If `y < 0`, the result is undefined.
      * If `y = 0`, the result is 1.
      * If `y > 0`, the result is 0.
  3. Otherwise, write `y` (which is rational) as a fraction: `a/b` or `-a/b`,
    where `a` and `b` are positive, [coprime](https://en.wikipedia.org/wiki/Coprime_integers) numbers.
  4. Let `r` be the `b`'th root of `x`.<br>
    Sometimes, `x` will have multiple roots, for example, 2 and -2 are both square roots of 4.
      * If all those roots are irrational, the result is undefined.
      * If there is exactly one rational root, let `r` be that root.
      * If there are two rational roots, let `r` be the one that is positive.
  6. If `y` is positive, the result is `r^a`.
  7. Otherwise, the result is `1 / r^a`.
- You can define multiple constants or functions in the same line, e.g. `one = uno = 1` or `f(x) = g(x) = x ^ 2`.
  * You can even do weird things like `divide(x, y) = reverseDivide(y, x) = x / y`.
  * You can even do weird things like: `f(x) = c = 1`.
- Computations and definitions must be written in a single line. For example, this definition is incorrect:

      x = 1 +
        3

- When loading a file, newlines are ignored inside comments and all types of brackets, for example, this definition is correct:

      x = (1 +
        3)

- You can't define two constants, functions, and/or function parameters with the same name.
- The exact [evaluation strategy](https://en.wikipedia.org/wiki/Evaluation_strategy) of recalc is unspecified.
  However, recalc is guaranteed evaluate expressions in such a way that
  it will never get stuck in an infinite loop if there's a way to avoid it.
- Constants are only evaluated the first time you try to get their value.
- Differences between interactive mode and files.<br>
  Loading a file is similar in principle to entering each of its lines in interactive mode, one by one -
  with the following exceptions:
  * In interactive mode, each line may only refer to constants and functions defined in previous lines or in the same line
    (in particular, a function may refer to itself).
    In a file, they may also refer to constants and functions defined after them, in the same file.
  * In interactive mode, you may overwrite constants and functions you have previously defined, e.g.

        f(x) = x^2
        g(x) = f(x) + 1
        f(x) = x + 1

    Here, we define `f(x)`, then use it to define `g(x)`, and then define another function called `f(x)`.
    The definition of `g(x)` will still use the old version of `f(x)`.<br>
    In a file, every definition must be unique within that file.
  * Files may not include special commands, like `:load <file>` or `:quit`.
  * In interactive mode, each computation, definition or comment must fit in a single line.
    In a file, newline characters are ignored inside brackets and comments.
