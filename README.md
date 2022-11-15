# Friendly Vector Format

## Values

## 1. Ergonomic
It is easy to read and to write FVF using a text editor. 

## 2. Simple to implement
It is simple to implement renderers and parsers for FVF.

## 3. Compatible
It is convertable to SVG and to common raster formats.


## Notes:

Good Example: https://github.com/dzhu/openscad-language-server/blob/master/src/main.rs

## TODO:

- Get Syntax highlighting to work with helix -> https://tree-sitter.github.io/tree-sitter/syntax-highlighting ✅
- Render using egui ✅
- Simple LSP Server ✅
- Show the syntax tree in Egui ✅
- store source code in LSP ✅
- Handle incremental updates ✅
- Draw shapes in egui
- Implement a simple interpreter

  - simple eval
  - simple environment
- First render in egui 
- Add syntax for defining functions
- Adopt eval for user-defined functions