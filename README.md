[Rust]: https://www.rust-lang.org/
[LLVM]: https://llvm.org/
[Inkwell]: https://github.com/TheDan64/inkwell
[Colored]: https://crates.io/crates/colored

<div align="center">
 <img src="https://custom-icon-badges.demolab.com/badge/-Toy-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/-Programming-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/-Language-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/-0.2.1-blue?style=for-the-badge&logoColor=white" />
</div>

### 👀 Description
**Toy Programming Language** - is a simple compiling language, based on LLVM. </br>
Project created to learn and show other people how to create compilers in Rust 🦀

In that case we can create variables with `int`, `bool` and `str` types, use binary operations on numbers and print them by `print()` function.
Compiler supports multiple error handling at each stage

### 🤖 Tools Used
* Programming Language: [Rust]
* Code Generator: [LLVM]
* LLVM Library: [Inkwell]
* Colored Terminal Library: [Colored]

### 🦛 Building
1. Download or clone this repository to your computer
2. Install **[Rust]** language
3. Type build command at the main directory:
```sh
cargo build --release
```
4. Binary file of compiler will be at `target/release` directory under the name: _**tplc**_ (or _**tplc.exe**_ on Windows)

### 👾 Example
1. Create file `example.tpl` and open in any code editor
2. Write code:
```c++
int a = 2; // annotation
int b = a * 2; // annotation using other variables
int c = 2 + 2 * 2; // binary operations priority

print(a); // 2
print(b); // 4
print(c); // 6

a = 2 + 2; // assignment
print(a); // 4

bool flag = true; // boolean type
print(flag); // will print "true"

str greeting = "Hello World!"; // string type
print(greeting); // "Hello World!"
```
3. Compile it by command:
```sh
tplc example.tpl output
```
4. And run like binary file:
```sh
./output
```

### 💀 License
Project licensed under the BSD-3 License. More information in [**LICENSE**](https://github.com/mealet/tpl-lang/blob/main/LICENSE) file
