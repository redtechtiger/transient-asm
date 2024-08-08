# Transient

## 🔥 ~ Getting started ~
> [!IMPORTANT]\
> This is a purely education project and consequently offers no guarantees. My projects are free to use for anyone so long as the relevant licenses are followed and credit is given, where appropriate. Enjoy!

This is a collection of utilities for compiling and running TransientAssembly (TASM) programs. The Transient project includes a compiler, which compilies TransientAssembly (TASM) down to TransientBytecode (TBC), which is then executed by the TransientVM (TVM).

#### ⬇️ Installation

The compiler (transientcompile) and virtual machine (transientvm) can both be installed as a bundle using Cargo. You first need to clone the GitHub repo:
```
$ git clone https://github.com/redtechtiger/transient
```
Then, navigate into the newly created directory and run Cargo to install the binaries:
```
$ cd transient
$ cargo install --path .
```

#### 🗑️ Removal

If you wish to remove these utilities from your system you can do so using Cargo. Run the following command:
```
$ cargo uninstall --package transient
```

#### 📚 Sample usage

To compile and run a TransientAssembly (TASM) file, run the following command, replacing _input.tasm_ with your input and _output.tbc_ if desired:
```
$ transientcompile input.tasm output.tbc
$ transientvm output.tbc
```
In order to test that the toolchain is working, you can try to compile of the example projects. For instance, here's how to compile the fibonacci sequence:
```
$ transientcompile examples/fibonacci.tasm fibonacci.tbc
$ transientvm fibonacci.tbc
```

## 💻 ~ TransientAssembly ~
This is a basic tutorial to get you started with writing TransientAssembly. As this entire project is in its early days, expect major changes to both syntax, features, and even the fundamental workings on the language. This also means that the language is very basic as of now, and may only make sense for those who are familiar with assembly or very low level code.
#### Structure
Every line in TransientAssembly is an instruction, with the exception of comments "//", tags "#", and empty lines. We'll get back to tags in a second, but for now, let's look at the general structure of the language. At the beginning of an instruction, you have the operation - pretty self explanatory once you read some of the examples. It's accompanied by a suffix (64, 32, 16, 8) that dictates how many bits the operation should operate on. Again, more on this later. You then have the arguments, which can be either a variable (prefixed by a `$`), or an intermediate (prefixed by a `!{size}_`, e.g. `!64_`).
```
// This is a comment.
// Blablabla

// This is a tag!
#tag_1

// And this is an operation. We are creating a 64 bit variable called my_variable with the value 5.
set64 $my_variable 5
```
#### Operations
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

#### Sizes
Every operation must include a size hint, however not all operations actually use it. For instance, the halt instruction (`hlt`) is always going to be 64 bits (8 bytes) but the compiler still requires you to specify a size for technical reasons. For all operations, you may keep this at 64 if you don't know what it does. Most importantly, this needs to be kept constant for the variables that concern it. For instance, you cannot create a 64 bit variable and attempt to use it in 32, 16, or 8 bit operations unless you really know what you're doing, or this will cause corruption of memory and/or runtime code, which will lead to nasty bugs. Currently the compiler does not validate this.

#### Tags
Tags are simply pointers to an instruction in the binary, useful when jumping. But fear not, for the compiler will calculate & produce these automatically, so the usage is very simple. Here's an example of an infinite loop:
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
jne64 #loop

hlt64
```
