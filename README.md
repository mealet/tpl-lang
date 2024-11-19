[Rust]: https://www.rust-lang.org/
[LLVM]: https://llvm.org/
[Inkwell]: https://github.com/TheDan64/inkwell
[Colored]: https://crates.io/crates/colored
[Examples]: ./examples

<div align="center">
 <img src="https://github.com/user-attachments/assets/291c4d80-e255-4c17-8543-8528e1a4ddda" /> </br>

 <img src="https://custom-icon-badges.demolab.com/badge/written_on-rust-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/based_on-llvm-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/version-0.3.3-blue?style=for-the-badge&logoColor=white" />
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
2. Install the compiler by:
```
cargo install --git https://github.com/mealet/tpl-lang
```

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
 You can also check snapped code in <a href="./examples">Examples</a>

 ### Boolean Operations
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

### Compares
```c
int32 a = 5;
int32 b = 10;

if a < b {
 print("less!");
} else {
 print("bigger");
};

// "less"
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

 for i index {
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

</details>

## 🤔 Others
#### Contributing
Check out our [**Contribution Guide**](CONTRIBUTING.md)

#### License
Project licensed under the BSD-3 License. More information in [**LICENSE**](LICENSE) file
