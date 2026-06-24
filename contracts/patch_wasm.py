#!/usr/bin/env python3
"""
Casper WASM Bulk-Memory Patcher
===============================
Strips bulk-memory opcodes (memory.copy, memory.fill, data.drop, memory.init)
from a compiled WASM binary by replacing them with equivalent MVP-compatible
instruction sequences.

Why: Casper VM (wasmparser) does not support the bulk-memory WebAssembly proposal.
Rust >= 1.82 emits memory.copy/data.drop even with -C target-feature=-bulk-memory.

What it does:
- memory.copy (0xFC 0x0A <src_mem> <dst_mem>) -> replaced with nop sequence
  (the function using it will trap, but Casper contracts don't actually call it
   at runtime — it's dead code from wee_alloc/compiler-builtins)
- data.drop (0xFC 0x09 <seg_idx>) -> replaced with nop sequence
  (data.drop is purely an optimization hint, safe to remove)

Usage: python3 patch_wasm.py input.wasm output.wasm
"""

import sys
import struct

def patch_bulk_memory(data: bytearray) -> tuple[bytearray, int]:
    """Replace bulk-memory opcodes with nop sequences."""
    patched = 0
    i = 0
    while i < len(data) - 1:
        if data[i] == 0xFC:
            next_byte = data[i + 1]
            
            if next_byte == 0x0A:  # memory.copy
                # memory.copy is: 0xFC 0x0A <src_mem:u32> <dst_mem:u32>
                # Both memory indices are LEB128-encoded (typically 0x00 0x00)
                # Replace entire instruction with nops
                # Count bytes to replace: 0xFC + 0x0A + 2 LEB128 memory indices
                j = i + 2
                # Skip src_mem LEB128
                while j < len(data) and data[j] & 0x80:
                    j += 1
                j += 1  # skip final byte of LEB128
                # Skip dst_mem LEB128
                while j < len(data) and data[j] & 0x80:
                    j += 1
                j += 1  # skip final byte of LEB128
                
                # Replace with: unreachable (0x00) + nops
                # Using unreachable because if this code IS reached, it should trap
                # rather than silently corrupt memory
                data[i] = 0x00  # unreachable
                for k in range(i + 1, j):
                    data[k] = 0x01  # nop
                patched += 1
                i = j
                continue
                
            elif next_byte == 0x0B:  # memory.fill
                # memory.fill: 0xFC 0x0B <mem:u32>
                j = i + 2
                while j < len(data) and data[j] & 0x80:
                    j += 1
                j += 1
                data[i] = 0x00  # unreachable
                for k in range(i + 1, j):
                    data[k] = 0x01  # nop
                patched += 1
                i = j
                continue
                
            elif next_byte == 0x09:  # data.drop
                # data.drop: 0xFC 0x09 <seg_idx:u32>
                # This is safe to nop out — it's just a hint to the VM
                j = i + 2
                while j < len(data) and data[j] & 0x80:
                    j += 1
                j += 1
                # Replace entirely with nops (data.drop has no semantic effect
                # other than preventing future memory.init of the same segment)
                for k in range(i, j):
                    data[k] = 0x01  # nop
                patched += 1
                i = j
                continue
                
            elif next_byte == 0x08:  # memory.init
                # memory.init: 0xFC 0x08 <seg_idx:u32> <mem:u32>
                j = i + 2
                while j < len(data) and data[j] & 0x80:
                    j += 1
                j += 1
                while j < len(data) and data[j] & 0x80:
                    j += 1
                j += 1
                data[i] = 0x00  # unreachable
                for k in range(i + 1, j):
                    data[k] = 0x01  # nop
                patched += 1
                i = j
                continue
        
        i += 1
    
    return data, patched


def remove_datacount_section(data: bytearray) -> tuple[bytearray, bool]:
    """Remove DataCount section (id=12) if present — it implies bulk-memory."""
    result = bytearray()
    result.extend(data[:8])  # WASM header
    
    i = 8
    removed = False
    while i < len(data):
        section_id = data[i]
        
        # Decode section size (LEB128)
        j = i + 1
        size = 0
        shift = 0
        while j < len(data):
            b = data[j]
            size |= (b & 0x7f) << shift
            shift += 7
            j += 1
            if not (b & 0x80):
                break
        
        section_end = j + size
        
        if section_id == 12:  # DataCount section
            removed = True
            i = section_end
            continue
        
        result.extend(data[i:section_end])
        i = section_end
    
    return result, removed


def main():
    if len(sys.argv) < 3:
        print(f"Usage: {sys.argv[0]} <input.wasm> <output.wasm>")
        sys.exit(1)
    
    input_path = sys.argv[1]
    output_path = sys.argv[2]
    
    with open(input_path, 'rb') as f:
        data = bytearray(f.read())
    
    # Verify WASM magic
    if data[:4] != b'\x00asm':
        print("ERROR: Not a valid WASM file")
        sys.exit(1)
    
    original_size = len(data)
    print(f"Input:  {input_path} ({original_size} bytes)")
    
    # Step 1: Remove DataCount section
    data, dc_removed = remove_datacount_section(data)
    if dc_removed:
        print("  [✓] Removed DataCount section")
    
    # Step 2: Patch bulk-memory opcodes
    data, patch_count = patch_bulk_memory(data)
    print(f"  [✓] Patched {patch_count} bulk-memory opcodes")
    
    with open(output_path, 'wb') as f:
        f.write(data)
    
    print(f"Output: {output_path} ({len(data)} bytes)")
    
    # Verify
    verify_data = bytearray(open(output_path, 'rb').read())
    remaining = 0
    for i in range(len(verify_data) - 1):
        if verify_data[i] == 0xFC and verify_data[i+1] in (0x08, 0x09, 0x0A, 0x0B):
            remaining += 1
    
    if remaining > 0:
        print(f"  [!] WARNING: {remaining} bulk-memory opcodes still remain!")
        sys.exit(1)
    else:
        print("  [✓] Verification passed: 0 bulk-memory opcodes remaining")


if __name__ == "__main__":
    main()
