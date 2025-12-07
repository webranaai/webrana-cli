;; ============================================
;; Hello World Plugin - WebAssembly Text Format
;; Created by: CIPHER (Team Beta)
;; ============================================
;; 
;; This is a simple WASM plugin demonstrating:
;; - Exported functions
;; - Basic arithmetic
;; - Memory management basics
;;
;; Compile with: wat2wasm plugin.wat -o plugin.wasm

(module
  ;; Memory export (1 page = 64KB)
  (memory (export "memory") 1)

  ;; Simple function: returns 42 (the answer)
  (func (export "greet") (result i32)
    i32.const 42
  )

  ;; Add two numbers
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Multiply two numbers
  (func (export "multiply") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; Get version (returns 100 for v1.0.0)
  (func (export "version") (result i32)
    i32.const 100
  )
)
