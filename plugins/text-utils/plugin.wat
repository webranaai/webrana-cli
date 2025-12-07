;; ============================================
;; Text Utils Plugin - WebAssembly Module
;; Created by: CIPHER (Team Beta)
;; ============================================

(module
  (memory (export "memory") 1)

  ;; Allocator pointer
  (global $heap (mut i32) (i32.const 4096))

  ;; Allocate n bytes
  (func (export "alloc") (param $size i32) (result i32)
    (local $ptr i32)
    global.get $heap
    local.set $ptr
    global.get $heap
    local.get $size
    i32.add
    global.set $heap
    local.get $ptr
  )

  ;; String length (null-terminated)
  (func (export "length") (param $ptr i32) (result i32)
    (local $len i32)
    i32.const 0
    local.set $len
    
    block $done
      loop $count
        local.get $ptr
        local.get $len
        i32.add
        i32.load8_u
        i32.eqz
        br_if $done
        
        local.get $len
        i32.const 1
        i32.add
        local.set $len
        
        br $count
      end
    end
    
    local.get $len
  )

  ;; Count char occurrences
  (func (export "count_chars") (param $ptr i32) (param $target i32) (result i32)
    (local $count i32)
    (local $i i32)
    (local $char i32)
    
    i32.const 0
    local.set $count
    i32.const 0
    local.set $i
    
    block $done
      loop $scan
        local.get $ptr
        local.get $i
        i32.add
        i32.load8_u
        local.set $char
        
        local.get $char
        i32.eqz
        br_if $done
        
        local.get $char
        local.get $target
        i32.eq
        if
          local.get $count
          i32.const 1
          i32.add
          local.set $count
        end
        
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        br $scan
      end
    end
    
    local.get $count
  )

  ;; To uppercase (in-place)
  (func (export "to_upper") (param $ptr i32) (result i32)
    (local $i i32)
    (local $char i32)
    
    i32.const 0
    local.set $i
    
    block $done
      loop $convert
        local.get $ptr
        local.get $i
        i32.add
        i32.load8_u
        local.set $char
        
        local.get $char
        i32.eqz
        br_if $done
        
        ;; If lowercase a-z (97-122), convert
        local.get $char
        i32.const 97
        i32.ge_u
        if
          local.get $char
          i32.const 122
          i32.le_u
          if
            local.get $ptr
            local.get $i
            i32.add
            local.get $char
            i32.const 32
            i32.sub
            i32.store8
          end
        end
        
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        br $convert
      end
    end
    
    local.get $i
  )

  ;; To lowercase (in-place)
  (func (export "to_lower") (param $ptr i32) (result i32)
    (local $i i32)
    (local $char i32)
    
    i32.const 0
    local.set $i
    
    block $done
      loop $convert
        local.get $ptr
        local.get $i
        i32.add
        i32.load8_u
        local.set $char
        
        local.get $char
        i32.eqz
        br_if $done
        
        ;; If uppercase A-Z (65-90), convert
        local.get $char
        i32.const 65
        i32.ge_u
        if
          local.get $char
          i32.const 90
          i32.le_u
          if
            local.get $ptr
            local.get $i
            i32.add
            local.get $char
            i32.const 32
            i32.add
            i32.store8
          end
        end
        
        local.get $i
        i32.const 1
        i32.add
        local.set $i
        
        br $convert
      end
    end
    
    local.get $i
  )

  ;; Reverse string (in-place)
  (func (export "reverse") (param $ptr i32) (param $len i32) (result i32)
    (local $left i32)
    (local $right i32)
    (local $temp i32)
    
    local.get $len
    i32.const 2
    i32.lt_s
    if
      i32.const 1
      return
    end
    
    i32.const 0
    local.set $left
    local.get $len
    i32.const 1
    i32.sub
    local.set $right
    
    block $done
      loop $swap
        local.get $left
        local.get $right
        i32.ge_s
        br_if $done
        
        ;; Swap
        local.get $ptr
        local.get $left
        i32.add
        i32.load8_u
        local.set $temp
        
        local.get $ptr
        local.get $left
        i32.add
        local.get $ptr
        local.get $right
        i32.add
        i32.load8_u
        i32.store8
        
        local.get $ptr
        local.get $right
        i32.add
        local.get $temp
        i32.store8
        
        local.get $left
        i32.const 1
        i32.add
        local.set $left
        
        local.get $right
        i32.const 1
        i32.sub
        local.set $right
        
        br $swap
      end
    end
    
    i32.const 1
  )

  ;; Check palindrome
  (func (export "is_palindrome") (param $ptr i32) (param $len i32) (result i32)
    (local $left i32)
    (local $right i32)
    
    local.get $len
    i32.const 2
    i32.lt_s
    if
      i32.const 1
      return
    end
    
    i32.const 0
    local.set $left
    local.get $len
    i32.const 1
    i32.sub
    local.set $right
    
    block $not_palindrome
      block $done
        loop $check
          local.get $left
          local.get $right
          i32.ge_s
          br_if $done
          
          local.get $ptr
          local.get $left
          i32.add
          i32.load8_u
          local.get $ptr
          local.get $right
          i32.add
          i32.load8_u
          i32.ne
          br_if $not_palindrome
          
          local.get $left
          i32.const 1
          i32.add
          local.set $left
          
          local.get $right
          i32.const 1
          i32.sub
          local.set $right
          
          br $check
        end
      end
      i32.const 1
      return
    end
    i32.const 0
  )

  ;; Is digit (0-9)
  (func (export "is_digit") (param $char i32) (result i32)
    local.get $char
    i32.const 48
    i32.ge_u
    if (result i32)
      local.get $char
      i32.const 57
      i32.le_u
    else
      i32.const 0
    end
  )

  ;; Is alpha (a-z, A-Z)
  (func (export "is_alpha") (param $char i32) (result i32)
    local.get $char
    i32.const 65
    i32.ge_u
    if (result i32)
      local.get $char
      i32.const 90
      i32.le_u
      if (result i32)
        i32.const 1
      else
        local.get $char
        i32.const 97
        i32.ge_u
        if (result i32)
          local.get $char
          i32.const 122
          i32.le_u
        else
          i32.const 0
        end
      end
    else
      i32.const 0
    end
  )
)
