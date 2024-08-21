[Rust]: https://www.rust-lang.org/
[LLVM]: https://llvm.org/
[Inkwell]: https://github.com/TheDan64/inkwell
[Colored]: https://crates.io/crates/colored

<div align="center">
 <img src="https://custom-icon-badges.demolab.com/badge/-Toy-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/-Programming-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/-Language-blue?style=for-the-badge&logoColor=white" />
 <img src="https://custom-icon-badges.demolab.com/badge/-0.2.9-blue?style=for-the-badge&logoColor=white" />
</div>

### 👀 Description
**Toy Programming Language** - is a simple compiling language, based on LLVM. </br>
Project created to learn and show other people how to create compilers in Rust 🦀

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
 ![image](https://github.com/user-attachments/assets/96f71bf9-bc11-4afa-b7ad-a3e81731d43e) </br>
 ![image](https://github.com/user-attachments/assets/ca948e3d-8398-4d82-b923-8d01e89a5b5b) </br>
 ![image](https://github.com/user-attachments/assets/523264db-ae4f-4c2f-b7a4-b13076461cf5) </br>
 ![image](https://github.com/user-attachments/assets/f18f6a49-a4ee-414b-8c2f-70e6345c94ff) </br>


</details>

### 💀 License
Project licensed under the BSD-3 License. More information in [**LICENSE**](https://github.com/mealet/tpl-lang/blob/main/LICENSE) file
