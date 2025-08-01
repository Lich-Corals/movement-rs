# Movement-RS

## What is this?
A small project I created to get more familiar with Rust. It is not designed to be useful for any specific purpose, besides learning Rust and having fun.
<br/>The main feature of the program is recording movement of the cursor and trying to find out if the recording is a circle, an ellipse or a straight line.

I'm not very familiar with the mathematical principles normally used to identify shapes from mere coordinate sets. Thus, this is just an attempt of determining which shape has been drawn using my limited knowledge.

If you happen to have a bit of time to spare to improve the program, you're welcome to do so and submit a pull-request and explain what you've done.
There are a few tests at the end of the file to check if pre-recorded shpes are recognized as the correct shape. Those tests are also connected with a GitHub action, which should automatically run if a pull-request is submitted.

## How does it work?
I explained this in detail in the [feature guide](https://github.com/Lich-Corals/movement-rs/blob/mistress/latex/feature_guide.pdf).

## How to use it?
1. Install Cargo (the package manager for rust)
2. download the project (using git) and cd into the directory:
```bash
git clone https://github.com/Lich-Corals/movement-rs.git && cd movement-rs
```
3. Compile the project:
```bash
cargo build --release
```
> [!NOTE]
> You don't have to use `--release`; but with this option it is optimized and without debug info.
> If you compile a development version, the next command is `./target/debug/movement`.

4. Run the executable:
```bash
./target/release/movement
```

> [!NOTE]
> I don't have any idea if and how this works on Windows or macOS...  
> The commands above are probably only working in a Linux shell.
>
> Besides that, the mouse position tracking in Wayland is a bit weïrd sometimes. In this case, the program is only able to track the cursor position while the mouse is above certain windows.
