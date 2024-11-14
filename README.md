[Rust]: https://www.rust-lang.org/
[LLVM]: https://llvm.org/
[Inkwell]: https://github.com/TheDan64/inkwell
[Colored]: https://crates.io/crates/colored

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

## 🦛 Building
1. Download or clone this repository to your computer.
2. Install **[Rust]** language.
3. Install **[LLVM]** for your system.
4. Type build command at the main directory:
```sh
cargo build --release
```
4. Binary file of compiler will be at `target/release` directory under the name: _**tplc**_ (or _**tplc.exe**_ on Windows)

## 👾 Example
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

// if-else construction

if 1 < 2 {
    print("1 is less than 2");
};

if 2 != 2 {
    // code
} else {
    print("2 = 2");
};

// loops
int a = 0;

while a < 5 {
    a += 1;
    // or
    a++;
    print(a);
};

for i in 5 {
    print(i);
};

// tests in variables
bool test = 1 + 1 == 2;

// defining functions
define int foo(int a, int b) {
    print("hello from foo function!");
    return a + b;
};

// calling functions
foo(4, 2);

// calling functions in variables annotation or assignment
int a = foo(4, 2);
a = foo(5, 5);
```
3. Compile it by command:
```sh
tplc example.tpl output
```
4. And run like binary file:
```sh
./output
```

<details>
 <summary><h2>😵 Errors Examples</h2></summary>

 ![image](https://github.com/user-attachments/assets/dca42b0f-dc68-4192-82d0-ae7523248b43) </br>
 ![image](https://github.com/user-attachments/assets/ca948e3d-8398-4d82-b923-8d01e89a5b5b) </br>
 ![image](https://github.com/user-attachments/assets/523264db-ae4f-4c2f-b7a4-b13076461cf5) </br>
 ![image](https://github.com/user-attachments/assets/68531892-8f89-42db-8831-2158ecbedc1a) </br>
 ![image](https://github.com/user-attachments/assets/82bb95e9-a342-447b-9b84-a9b31bfe636d) </br>
 ![image](https://github.com/user-attachments/assets/8f95565c-7ca2-4728-986f-c1eb990b3602) </br>
 ![image](https://github.com/user-attachments/assets/815b7acb-917d-49a8-995a-85e02035b5f2) </br>

</details>

## 🤔 Contributing
Check out our [**Contribution Guide**](CONTRIBUTING.md)

## 💀 License
Project licensed under the BSD-3 License. More information in [**LICENSE**](LICENSE) file
