# Pact design

## File format

Pact binary files end in the `.pact` extension. Pact assembly files end in `.pasm`.

Pact binary files begin with the two-byte magic number `0x8bca`.

## Instructions

### Instruction layout

Immediate (imm):

```
opcode      operand
/-----\  /-----------\
|     |  |           |
0  1  2  3  4  5  6  7
```

Register (reg):

```
 use value as register id
\-------\-/--------------/
         V
opcode   I  src   dest
/-----\  +  /--\  /--\
|     |  |  |  |  |  |
0  1  2  3  4  5  6  7
```

Memory (mem):

```
 use value as pointer
\-------\-/----------/
         V
opcode   P   address
/-----\  +  /--------\
|     |  |  |        |
0  1  2  3  4  5  6  7
```

*Note:* memory functions append address operand to rd to obtain the final address.

I/O (i/o):

```
        function id
       \-----------/
                 |
  device id      |
 \-------\-/    \-/
          V      V
opcode   dev    func
/-----\  /--\  /-----\
|     |  |  |  |     |
0  1  2  3  4  5  6  7
```

### Instruction set

- `000 : ADI (imm)` Add a 5-bit immediate to ra. Sets flags.
- `001 : ADD (reg)` Add src to dest. Sets flags.
- `010 : SUB (reg)` Subtract src from dest. Sets flags.
- `011 : JNE (reg)` Jump if flags `!Z`.
- `100 : JG  (mem)` Jump if flags `S`.
- `101 : JL  (mem)` Jump if flags `!S && !Z`.
- `110 : IOI (i/o)` Perform an I/O operation with ra.
- `111 : IOR (i/o)` Perform an I/O operation, interpreting ra as a register ID.

## Registers

- `00 : ra` Accumulator / return value.
- `01 : rb` General-purpose.
- `10 : rc` Stack pointer.
- `11 : rd` High eight bits of address.

## Memory

12-bit memory space, addressable via `rd:operand`. Not certain yet whether or not program and data are mixed.

First 8 bytes of data should be reserved for swap (register storage).

## I/O

Four hardware "devices" are available at any time. The first is always the CPU.

The following are the four default devices:

- `00 - cpu` The computer itself.
- `01 - kbd` The input (keyboard).
- `10 - scr` The output (screen).
- `11 - mth` The extended ALU.

*Note:* neither the keyboard nor screen have standard implementations. They may be text-based or otherwise.

### CPU

- `000 : hlt` Halt the computer.
- `001 : sds` Set device section.
- `010 : gds` Return current device section.
- `011 : lod` Load a value from `rd:operand` into ra.
- `100 : str` Store value in ra into `rd:operand`.
- `101 : lda` Load a value from the address at `rd:operand` into ra.
- `110 : sta` Store value in ra into the address at `rd:operand`.
- `111 : res` Reserved.

### Keyboard

- `000 : scn` Scan for input, returning 0 if none.
- `001 : blk` Block on input.
- `010 : cf1` Configuration option 1.
- `011 : cf2` Configuration option 2.

### Screen

- `000 : stx` Set X coordinate.
- `001 : sty` Set Y coordinate.
- `010 : dis` Display byte at X, Y.
- `011 : gtx` Query X coordinate.
- `100 : gty` Query Y coordinate.
- `101 : clr` Clear the screen.
- `110 : cf1` Configuration option 1.
- `111 : cf2` Configuration option 2.

### Extended ALU

*Note:* all operations set flags.

- `000 : mul` Multiply ra by operand, storing result in `rb:ra`.
- `001 : div` Integer divide ra by operand.
- `010 : and` Bitwise AND ra and operand.
- `011 : bor` Bitwise OR ra and operand.
- `100 : xor` Bitwise XOR ra and operand.
- `101 : neg` Returns the negative of the number via two's complement.
- `110 : gfl` Returns the flags in form `000000SZ`.
- `111 : sfl` Sets the flags in form `000000SZ`.

### Custom devices

While the defaults specify the basic devices, you will probably want more. You can use more devices via device management,
which exposes 256 device sections of 4 each (the first, 0x00, being the default devices). However, the first device will
always be the CPU, since it is necessary for managing the device sections.

When using special devices, it is ideal to place input-oriented devices in the `01` slot, and output-oriented devices in
the `10` slot, with operations or instruction supplements in the `11` slot.

Any unspecified functions on a standard device may be customized. However, do not customize reserved functions.

