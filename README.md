# COBS-RS

A very minimal no_std [Consistent Overhead Byte
Stuffing](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing) library.

## Usage

This library provides 2 functions.

`stuff` and `unstuff` which encode and decode according to the COBS standard,
respectively.

## Example

```rust
let data: [u8; 254] = [
    // ...snip
    # 0; 254
];

// Encode the data
let encoded: [u8; 256] = cobs_rs::stuff(data, 0x00);

// ... snip

// Decode the data
let decoded: [u8; 254] = cobs_rs::unstuff(encoded, 0x00);

assert_eq!(data, decoded);
```

## License

Licensed under a __MIT__ license.
