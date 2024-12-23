[Rust]: https://www.rust-lang.org/
[LLVM]: https://llvm.org/
[Inkwell]: https://github.com/TheDan64/inkwell
[Colored]: https://crates.io/crates/colored
[Examples]: ./examples
[Releases]: https://github.com/mealet/tpl-lang/releases

<div align="center">
 <img src="https://github.com/user-attachments/assets/291c4d80-e255-4c17-8543-8528e1a4ddda" /> </br>

 <img src="https://tokei.rs/b1/github/mealet/tpl-lang?branch=main&style=for-the-badge&color=%230389f5" />
 <img src="https://img.shields.io/badge/dynamic/toml?url=https%3A%2F%2Fraw.githubusercontent.com%2Fmealet%2Ftpl-lang%2Frefs%2Fheads%2Fmain%2FCargo.toml%3Fraw%3Dtrue&query=workspace.package.version&style=for-the-badge&label=Version&color=%230389f5" />
</div>

## 🧐 What is this?
**Toy Programming Language** - is a simple compiling language, based on LLVM. </br>
Project created to learn and show other people how to create compilers in Rust 🦀

Code separated to 4 modules:
1. `tpl-lexer` - lexical analyzer, which turns code into _tokens_.
2. `tpl-parser` - tool for parsing tokens and creating _AST_.
3. `tpl-ir` - codegen module with simple task: translate _AST_ to _LLVM Module_.
4. `tplc` - main part of all project, which contains such things like: cli tool, config parser, llvm module compiler, object linker and etc.


## 🤖 Tools Used
* Programming Language: [Rust]
* Code Generator: [LLVM]
* LLVM Library: [Inkwell]
* Colored Terminal Library: [Colored]

## 💡 Installation
1. Install any of these C compilers: `clang`, `gcc`, `cc`.
2. Download archive for you'r system from [Releases]
3. Unpack it anywhere
4. Use the binary file (`tplc` on Linux/Mac, `tplc.exe` on Windows)

## 🦛 Building
1. Download or clone this repository to your computer.
2. Install **[Rust]** language.
3. Install **[LLVM]** for your system.
4. Type build command at the main directory:
```sh
cargo build --release
```
4. Binary file of compiler will be at `target/release` directory under the name: _**tplc**_ (or _**tplc.exe**_ on Windows)

<details>
 <summary><h2>👾 Examples</h2></summary>

### Types
```c
int8 // - 8 bit integer number
int16 // - 16 bit integer number
int32 // - 32 bit integer number
int64 // - 64 bit integer number

str // - string type
bool // - boolean type (true, false)
void // - void type (better for functions)
```

### Binary Operations
```c
int32 a = 10;
int32 b = 2;

print(a + b); // 12
print(a - b); // 8
print(a * b); // 20
```

### Defining Functions
```c
define int32 foo(int32 a, int32 b) {
 return a * b;
};

print(foo(5, 10)) // 50
```

### Boolean Operations
```c
int32 a = 5;
int32 b = 10;

if a < b {
 print("less!");
} else {
 print("bigger");
};

// "less"

// also supported
a < b
a > b
a == b
a != b
```

### Loops
```c
int32 counter = 0;

// while
while counter < 10 {
 print(counter);
 counter += 1;
};

// for
for count in 10 {
 print(count);
};
```

### Strings
```c
str a = "Hello";
str b = ", World!";

print(concat(a, b)); // Hello, World!
print(a, b); // Same as previous
```

### Lambda functions
```c
fn<int64> fib = int64 ( int64 index ) {
 int64 left = 0;
 int64 right = 1;
 int64 result = 0;

 for i in index {
  result = left + right;
  left = right;
  right = result;
 };

 return result;
};

int64 result = fib(1000);
print(result) // 9079565065540428013
```

### Boolean types
```c
bool a = true;
bool b = false;

print(a, b) // true false
```

### Pointers
```c
int32 a = 5;
int32* b = &a;

print(a); // 5

*b = 100;

print(a); // 100
```

### Sub-functions
```c
define int32 foo(int32 a) {
 return a * 2;
};

int32 value = 5;

print(value.foo()); // 10
```

### Arrays
```c
int32[5] a = [1, 2, 3, 4, 5];
// or
auto a = [1, 2, 3, 4, 5];

print(a); // [1, 2, 3, 4, 5];
```

### Type function
```c
int32 a = 5;
int8 b;
bool c = false;

print(a.type()); // int32
print(b.type()); // int8
print(c.type()); // bool
```

### Conversions
```
int32 a = 5;
int8 b = a.to_int8();
str c = b.to_str();

print(a.type()); // int32
print(b.type()); // int8
print(c.type()); // str
```

</details>

## 🤔 Others
#### Contributing
Check out our [**Contribution Guide**](CONTRIBUTING.md)

#### License
Project licensed under the BSD-3 License. More information in [**LICENSE**](LICENSE) file
