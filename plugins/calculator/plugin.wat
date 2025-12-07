;; ============================================
;; Calculator Plugin - WebAssembly Module
;; Created by: CIPHER (Team Beta)
;; ============================================

(module
  (memory (export "memory") 1)

  ;; Add: a + b
  (func (export "add") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.add
  )

  ;; Subtract: a - b
  (func (export "subtract") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.sub
  )

  ;; Multiply: a * b
  (func (export "multiply") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.mul
  )

  ;; Divide: a / b (returns -1 if b is 0)
  (func (export "divide") (param $a i32) (param $b i32) (result i32)
    local.get $b
    i32.eqz
    if (result i32)
      i32.const -1
    else
      local.get $a
      local.get $b
      i32.div_s
    end
  )

  ;; Modulo: a % b
  (func (export "modulo") (param $a i32) (param $b i32) (result i32)
    local.get $b
    i32.eqz
    if (result i32)
      i32.const -1
    else
      local.get $a
      local.get $b
      i32.rem_s
    end
  )

  ;; Power: base^exp (simple loop)
  (func (export "power") (param $base i32) (param $exp i32) (result i32)
    (local $result i32)
    (local $i i32)
    
    i32.const 1
    local.set $result
    i32.const 0
    local.set $i
    
    block $done
      loop $loop
        local.get $i
        local.get $exp
        i32.ge_s
        br_if $done
        
        local.get $result
        local.get $base
        i32.mul
        local.set $result
        
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        br $loop
      end
    end
    
    local.get $result
  )

  ;; Factorial: n!
  (func (export "factorial") (param $n i32) (result i32)
    (local $result i32)
    (local $i i32)
    
    local.get $n
    i32.const 1
    i32.le_s
    if (result i32)
      i32.const 1
    else
      i32.const 1
      local.set $result
      i32.const 2
      local.set $i
      
      block $done
        loop $loop
          local.get $i
          local.get $n
          i32.gt_s
          br_if $done
          
          local.get $result
          local.get $i
          i32.mul
          local.set $result
          
          local.get $i
          i32.const 1
          i32.add
          local.set $i
          
          br $loop
        end
      end
      
      local.get $result
    end
  )

  ;; Fibonacci: fib(n)
  (func (export "fibonacci") (param $n i32) (result i32)
    (local $a i32)
    (local $b i32)
    (local $temp i32)
    (local $i i32)
    
    local.get $n
    i32.const 0
    i32.le_s
    if (result i32)
      i32.const 0
    else
      local.get $n
      i32.const 1
      i32.eq
      if (result i32)
        i32.const 1
      else
        i32.const 0
        local.set $a
        i32.const 1
        local.set $b
        i32.const 2
        local.set $i
        
        block $done
          loop $loop
            local.get $i
            local.get $n
            i32.gt_s
            br_if $done
            
            local.get $a
            local.get $b
            i32.add
            local.set $temp
            
            local.get $b
            local.set $a
            
            local.get $temp
            local.set $b
            
            local.get $i
            i32.const 1
            i32.add
            local.set $i
            
            br $loop
          end
        end
        
        local.get $b
      end
    end
  )

  ;; Absolute value
  (func (export "abs") (param $n i32) (result i32)
    local.get $n
    i32.const 0
    i32.lt_s
    if (result i32)
      i32.const 0
      local.get $n
      i32.sub
    else
      local.get $n
    end
  )

  ;; Max of two numbers
  (func (export "max") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.gt_s
    if (result i32)
      local.get $a
    else
      local.get $b
    end
  )

  ;; Min of two numbers
  (func (export "min") (param $a i32) (param $b i32) (result i32)
    local.get $a
    local.get $b
    i32.lt_s
    if (result i32)
      local.get $a
    else
      local.get $b
    end
  )

  ;; Is even (1=true, 0=false)
  (func (export "is_even") (param $n i32) (result i32)
    local.get $n
    i32.const 2
    i32.rem_s
    i32.eqz
  )
)
