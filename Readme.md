# Kirin Lang

## Syntax

simple program

    package main

    native fn print_str(arg: string): void

    fn main(): void {
        print_str("Hello Kirin")
    }

if statements

    x : i64 = 400

    if x > 200 {
        print("x > 200")
    } else {
        print("x <= 200")
    }

loops

    for i := 0; i < 10; i++ {
        print(i)
    }