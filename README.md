# COBS-RS

A very minimal no_std [Consistent Overhead Byte
Stuffing](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing)
library written in Rust. The COBS algorithm, and thus also this crate, provides
an encoding for arbitrary data which removes any occurrence of a specific marker
byte. This is mostly useful when we are transferring arbitrary data which
is terminated with a null byte, and therefore we don't want our arbitrary data
buffer to contain any null bytes. In fact, this crate will automatically the
marker byte at the end of any encoded buffer.

## Features

The *cobs-rs* crate only provides two specific functions. Namely, the
[`stuff`](https://docs.rs/cobs-rs/1.0.2/cobs_rs/fn.stuff.html) and the
[`unstuff`](https://docs.rs/cobs-rs/1.0.2/cobs_rs/fn.unstuff.html) function,
which encode and decode respectively. This, together with the fact that the
crate doesn't use the [`std`](https://doc.rust-lang.org/std/), makes the crate
perfect for embedded hardware. However, it can also be used outside of embedded
systems.

## Usage

Both the encode([`stuff`](https://docs.rs/cobs-rs/1.0.2/cobs_rs/fn.stuff.html))
and decode([`unstuff`](https://docs.rs/cobs-rs/1.0.2/cobs_rs/fn.unstuff.html))
functions, use [const
generics](https://blog.rust-lang.org/2021/02/26/const-generics-mvp-beta). This
may make usage a bit counter-intuitive for people unfamiliar with this feature
at first.

Something to take into account here is that the COBS algorithm will __at most__
add `2 + (size of input buffer / 256)` (with integer division) bytes to the
encoded buffer in size compared to input buffer. This fact allows us to always
reserve enough space for the output buffer.

### Encoding buffers

Let us have a look at a small example of how to encode some data using the
[`stuff`](https://docs.rs/cobs-rs/1.0.2/cobs_rs/fn.stuff.html) function.

```rust
let data: [u8; 254] = [
    // ...snip
];

// Our input buffer is 254 bytes long.
// Thus, we need to reserve 2 + (254 / 256) = 2 extra bytes
// for the encoded buffer.
let encoded: [u8; 256] = cobs_rs::stuff(data, 0x00);

// We can also encode much larger buffers
let a_lot_of_data: [u8; 1337] = [
    // ...snip
];

// Our input buffer is 1337 bytes long.
// Thus, we need to reserve 2 + (1337 / 256) = 7 extra bytes
// for the encoded buffer.
let a_lot_of_output: [u8; 1344] = cobs_rs::stuff(a_lot_of_data, 0x00);
```

> **Note:** The output buffer type specifications are always necessary. The type
> specifications for the input data isn't necessary most of the time.

### Decoding buffers

Now, let us look at an example of how to decode data using the
[`unstuff`](https://docs.rs/cobs-rs/1.0.2/cobs_rs/fn.stuff.html) function.

It is generally a good idea to reserve `size of encoded buffer - 2` bytes for
the decoded buffer. With this rule, we will always have enough space for the
encoded buffer. Next to the decoded buffer, the
[`unstuff`](https://docs.rs/cobs-rs/1.0.2/cobs_rs/fn.stuff.html) function will
also return the actual filled size of the buffer.

```rust
// We are given some encoded data buffer
let encoded_data: [u8; 256] = [
    //... snip
];

// We reserve 256 - 2 = 254 bytes for the decoded buffer.
let (decoded_data: [u8; 254], decoded_data_length) =
    cobs_rs::unstuff(encoded_data, 0x00);

// We can also decode bigger buffers
let a_lot_of_encoded_data: [u8; 1344] = [
    //... snip
];

// We reserve 1344 - 2 = 1342 bytes for the decoded buffer.
let (a_lot_of_decoded_data: [u8; 1342], a_lot_of_decoded_data_length) =
    cobs_rs::unstuff(encoded_data, 0x00);
```

> **Note:** The decoded buffer type specifications are always necessary. The
> type specifications for the encoded data isn't necessary most of the time.

## License

Licensed under a __MIT__ license.
