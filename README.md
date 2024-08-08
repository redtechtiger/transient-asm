# Transient

## 🔥 ~ Getting started ~
> [!IMPORTANT]\
> This is a purely education project and consequently offers no guarantees. My projects are free to use for anyone so long as the relevant licenses are followed and credit is given, where appropriate. Enjoy!

This is a collection of utilities for compiling and running TransientAssembly (TASM) programs. The Transient project includes a compiler, which compilies TransientAssembly (TASM) down to TransientBytecode (TBC), which is then executed by the TransientVM (TVM). This document includes an installation guide, a guide to the compiler, and a quick tutorial of how to write TransientAssembly (TASM).

### ⬇️ Installation

> [!NOTE]\
> This guide assumes that you have a Rust toolchain installed. If you don't have this, install Rust using rustup or a suitable equivalent. A rust compiler is only needed when first installing the compiler and virtual machine. Their runtime do not rely on the rust toolchain.

The compiler (transientcompile) and virtual machine (transientvm) can both be installed as a bundle using Cargo. You first need to clone the GitHub repo:
```
$ git clone https://github.com/redtechtiger/transient
```
Then, navigate into the newly created directory and run Cargo to install the binaries:
```
$ cd transient
$ cargo install --path .
```

### 🗑️ Removal

If you wish to remove these utilities from your system you can do so using Cargo. Run the following command:
```
$ cargo uninstall --package transient
```

### 📚 Sample usage

To compile and run a TransientAssembly (TASM) file, simply invoke the compiler with the input and output file. This produces a .tbc file (Transient bytecode) which you can then run using the virtual machine. In order to test that the toolchain is working, you can try to compile of the example projects. For instance, here's how to compile the fibonacci sequence:
```
$ transientcompile examples/fibonacci.tasm fibonacci.tbc
$ transientvm fibonacci.tbc
```

## 💻 ~ TransientAssembly ~
This is a basic tutorial to get you started with writing TransientAssembly. As this entire project is in its early days, expect major changes to both syntax, features, and even the fundamental workings on the language. This also means that the language is very basic as of now, and may only make sense for those who are familiar with assembly or very low level code.
### Structure
Here's a typical instruction to have as a reference while you read the text below.
```
add64 $variable1 $another_var $result
```
Every line in TransientAssembly will execute one operation, with the exception of comments "//", tags "#", and empty lines. We'll get back to tags in a second, but for now, let's look at the general structure of the language. At the beginning of a line, you have the operation - pretty self explanatory once you read some of the examples. It's accompanied by a suffix (64, 32, 16, 8) that dictates how many bits the variables in the expression are. Again, more on this later. You then have the arguments, which can be either a variable (prefixed by a `$`), or an intermediate (prefixed by a `!{size}_`, e.g. `!64_`). Have a look at the following examples:
```
// This is a comment.
// Blablabla

// This is a tag!
#tag_1

// And this is an operation. We are creating a 64 bit variable called `my_variable` with the value 5.
set64 $my_variable 5

// Create another 64 bit variable with value 10
set64 $another_var 10

// We need to initialize this beforehand!
set64 $result 0

// Add `my_variable` and `another_var`, and store the sum in `result`.
add64 $my_variable $another_var $result

// The variable `result` will now be 15
```
### Operations
Here is a list of available operations.
```
mov - Copies the first variable into the second
add - Adds two variables and stores the result in a third
sub - Subtracts the second variable from the first and stores the result in a third
mov - Same except multiplied
divt - Same except divided (truncated)
divr - Same except divided (rounded)
rem - Same except moduli
cgt - Compares if the first variable is greater than the second variable. If true, the third variable will be set to 1. If false, it will be set to 0
cls - Same except less than
equ - Same except equals
jmp - Stops execution, jumps to a tag, and resumes
jie - If variable two is 1, jumps to a tag. If 0 or other value, keep executing as normal
jne - Same except only jumps if variable is 0
puti - Prints the integer stored at the first variable to the console
putc - Prints the integer at first variable formatted to an ascii character
imz - Get the size of the program in bytes and stores it in the first variable
hlt - Stop program execution and exit the virtual machine
```

### Sizes
The size of an operation just specifies how big the variables are that are used. For example, if you are adding two 8-bit integers, you **have** to use `add8`. The output of `add8` will also **always** be an 8-bit integer. Likewise, if you're adding two 64-bit integers and saving into a 64-bit integer, you **need** to use `add64`. Every operation must include a size hint, however not all operations actually use it. For instance, the halt instruction (`hlt`) is always going to be 64 bits (8 bytes) but the compiler still requires you to specify a size for technical reasons. 

Most importantly, the size **needs** to be the same for **all** the variables that concern it. (For all operations, you may keep this at 64 if you're unsure of what to choose). For instance, you cannot create a 64 bit variable and attempt to use it in 32, 16, or 8 bit operations unless you really know what you're doing, or this will cause corruption of memory and/or runtime code, which will lead to nasty bugs. Currently the compiler does not validate this, so it is up to you to verify this.

### Tags
Tags are simply points in your code that you can jump to. Under the hood, the compiler expands these to the memory address of the closest instruction after the tag. Here's an example of an infinite loop:
```
#foo
jmp #foo
```
Here's another loop that counts to 10.
```
set64 $value 0
set64 $comparison_result 0

#loop
puti64 $value
add64 $value !64_1 $value
cgt64 $value !64_10 $comparison_result
jne64 #loop $comparison_result

hlt64
```
