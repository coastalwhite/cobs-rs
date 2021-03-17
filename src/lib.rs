//! A very minimal no_std [Consistent Overhead Byte
//! Stuffing](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing)
//! library written in Rust. The COBS algorithm, and thus also this crate, provides
//! an encoding for arbitrary data which removes any occurrence of a specific marker
//! byte. This is mostly useful when we are transferring arbitrary data which
//! is terminated with a null byte, and therefore we don't want our arbitrary data
//! buffer to contain any null bytes. In fact, this crate will automatically the
//! marker byte at the end of any encoded buffer.
//!
//! ## Features
//!
//! The *cobs-rs* crate only provides two specific functions. Namely, the
//! [`stuff`] and the [`unstuff`] function, which encode and decode respectively. This, together
//! with the fact that the crate doesn't use the [`std`](https://doc.rust-lang.org/std/index.html),
//! makes the crate perfect for embedded hardware. However, it can also be used outside of embedded
//! systems.
//!
//! ## Usage
//!
//! Both the encode([`stuff`]) and decode([`unstuff`]) functions, use [const
//! generics](https://blog.rust-lang.org/2021/02/26/const-generics-mvp-beta). This
//! may make usage a bit counter-intuitive for people unfamiliar with this feature
//! at first.
//!
//! Something to take into account here is that the COBS algorithm will __at most__
//! add `2 + (size of input buffer / 256)` (with integer division) bytes to the
//! encoded buffer in size compared to input buffer. This fact allows us to always
//! reserve enough space for the output buffer.
//!
//! ### Encoding buffers
//!
//! Let us have a look at a small example of how to encode some data using the
//! [`stuff`] function.
//!
//! ```no_run
//! let data: [u8; 254] = [
//!     // ...snip
//! # 0; 254
//! ];
//!
//! // Our input buffer is 254 bytes long.
//! // Thus, we need to reserve 2 + (254 / 256) = 2 extra bytes
//! // for the encoded buffer.
//! let encoded: [u8; 256] = cobs_rs::stuff(data, 0x00);
//!
//! // We can also encode much larger buffers
//! let a_lot_of_data: [u8; 1337] = [
//!     // ...snip
//! # 0; 1337
//! ];
//!
//! // Our input buffer is 1337 bytes long.
//! // Thus, we need to reserve 2 + (1337 / 256) = 7 extra bytes
//! // for the encoded buffer.
//! let a_lot_of_output: [u8; 1344] = cobs_rs::stuff(a_lot_of_data, 0x00);
//! ```
//!
//! > **Note:** The output buffer type specifications are always necessary. The type
//! > specifications for the input data isn't necessary most of the time.
//!
//! ### Decoding buffers
//!
//! Now, let us look at an example of how to decode data using the [`unstuff`] function.
//!
//! It is generally a good idea to reserve `size of encoded buffer - 2` bytes for
//! the decoded buffer. With this rule, we will always have enough space for the
//! encoded buffer. Next to the decoded buffer, the [`unstuff`] function will
//! also return the actual filled size of the buffer.
//!
//! ```no_run
//! // We are given some encoded data buffer
//! let encoded_data: [u8; 256] = [
//!     //... snip
//! # 0; 256
//! ];
//!
//! // We reserve 256 - 2 = 254 bytes for the decoded buffer.
//! let (decoded_data, decoded_data_length): ([u8; 254], usize) =
//!     cobs_rs::unstuff(encoded_data, 0x00);
//!
//! // We can also decode bigger buffers
//! let a_lot_of_encoded_data: [u8; 1344] = [
//!     //... snip
//! # 0; 1344
//! ];
//!
//! // We reserve 1344 - 2 = 1342 bytes for the decoded buffer.
//! let (a_lot_of_decoded_data, a_lot_of_decoded_data_length): ([u8; 1342], usize) =
//!     cobs_rs::unstuff(encoded_data, 0x00);
//! ```
//!
//! > **Note:** The decoded buffer type specifications are always necessary. The
//! > type specifications for the encoded data isn't necessary most of the time.
//!
//! ## License
//!
//! Licensed under a __MIT__ license.

#![no_std]
#![warn(missing_docs)]

use core::convert::TryInto;

struct MarkerInfo {
    index: usize,
    points_to: usize,
}

impl MarkerInfo {
    fn adjust_accordingly<const SIZE: usize>(
        &mut self,
        out_buffer: &mut [u8; SIZE],
        new_index: usize,
    ) {
        out_buffer[self.index] = (new_index - self.index).try_into().unwrap();

        self.index = new_index;
        self.points_to = new_index + 0xff;
    }
}

/// Takes an input buffer and a marker value and COBS-encodes it to an output buffer.
///
/// Removes all occurrences of the marker value and adds one occurrence at the end. The returned
/// buffer should at least be 2 greater than the input buffer and for every roughly 256 bytes an
/// extra byte is conditionally added to the output buffer. All left-over space will and the end of
/// the buffer and will be filled with the marker value.
///
/// # Examples
///
/// ## Stuffing arbitrary data
///
/// ```
/// let transfer: [u8; 256] = cobs_rs::stuff(
///     *b"Hi everyone! This is a pretty nifty example.",
///     b'i'
/// );
///
/// // Now the data won't contain 'i's anymore except for the terminator byte.
/// # assert!(transfer[..45].into_iter().all(|byte| *byte != b'i'));
/// ```
///
/// ## Making sure there are no null bytes anymore
///
/// ```
/// let data = [
///     // ...snip
/// #       1
/// ];
///
/// let transfer: [u8; 256] = cobs_rs::stuff(data, 0x00);
///
/// // Now the data won't contain null bytes anymore except for the terminator byte.
/// ```
///
/// # Panics
///
/// This function panics, if the output buffer has too little space to fill the data from the input
/// buffer with.
///
pub fn stuff<const INPUT: usize, const OUTPUT: usize>(
    buff: [u8; INPUT],
    marker: u8,
) -> [u8; OUTPUT] {
    let mut output_buffer: [u8; OUTPUT] = [marker; OUTPUT];

    // Keep track of where the last marker was.
    // This always has one in the beginning, which is the overhead byte.
    let mut last_marker = MarkerInfo {
        index: 0,
        points_to: 0xff,
    };

    // Every time we set additional overhead marker, we should increase the offset.
    // This way we keep track what the relationship is between the input array indices and the
    // output array indices.
    let mut overhead_bytes = 1;

    // Loop through all the input bytes.
    for i in 0..INPUT {
        // Fetch the value of the input byte array.
        let value = buff[i];

        if last_marker.points_to == (overhead_bytes + i) {
            // Update the last marker and set the marker info to this new overhead byte.
            last_marker.adjust_accordingly(&mut output_buffer, overhead_bytes + i);

            // Say that we have another overhead byte.
            overhead_bytes += 1;
        }

        // If the current input value is a marker, adjust the previous marker accordingly and skip
        // the setting of the value, although it doesn't really matter.
        if value == marker {
            // Update the last marker value and info to this new marker.
            last_marker.adjust_accordingly(&mut output_buffer, overhead_bytes + i);

            continue;
        }

        // Update the output buffer value
        output_buffer[overhead_bytes + i] = value;
    }

    // For the last byte we update the previous marker.
    output_buffer[last_marker.index] = (INPUT + overhead_bytes - last_marker.index)
        .try_into()
        .unwrap();

    output_buffer
}

/// Takes an input buffer and a marker value and COBS-decodes it to an output buffer.
///
/// Removes all overhead bytes, inserts the marker where appropriate and __stops immediately__ when
/// a marker value is found. The size of output buffer is at least 2 bytes smaller than the size
/// of the input buffer. All left-over space will and the end of the buffer and will be filled with
/// the `0x00` bytes. The tuple returned contains both the decoded buffer and the actual filled
/// length of that buffer.
///
/// # Examples
///
/// ```no_run
/// let transferred_data: [u8; 258] = [
///     // ... snip
/// # 0; 258
/// ];
///
/// // We convert the COBS-encoded transferred_data to the plain data
/// // using the unstuff function.
/// let (plain_data, plain_data_length): ([u8; 256], usize) =
///     cobs_rs::unstuff(transferred_data, 0x00);
///
/// // ... snip
/// ```
///
/// # Panics
///
/// Panics if we don't have don't have a marker value in the input buffer.
///
/// This function panics if the output buffer has too little space to fill the data from the input
/// buffer with. This never happens if we reserve enough memory for the output, that being two less
/// bytes than the input buffer.
pub fn unstuff<const INPUT: usize, const OUTPUT: usize>(
    buff: [u8; INPUT],
    marker: u8,
) -> ([u8; OUTPUT], usize) {
    let mut output_buffer = [0; OUTPUT];

    // Keep track when the next marker will be. Initial this will be after the first overhead byte
    // value. We have to do minus 1 here, because we start our loop at 1 instead of 0.
    let mut until_next_marker = buff[0] - 1;
    // If this bits value is 0xff, we know that the next value will be an overhead byte, so keep
    // track of that.
    let mut next_is_overhead_byte = buff[0] == 0xff;

    // Keep track of the amount of overhead bytes, so that we can compensate for it when filling
    // our output buffer.
    let mut overhead_bytes = 1;

    // We can skip byte since it is the overhead byte we already know about.
    let mut i = 1;

    let output_buffer_length = loop {
        // Fetch the value from the input buffer.
        let value = buff[i];

        // If we value is the marker, we know we have reached the end.
        if value == marker {
            break i;
        }

        // If the current character is a marker or a overhead byte.
        if until_next_marker == 0 {
            // We know that the distance to the next marker will be the value of this marker.
            until_next_marker = value;

            // If this byte was a overhead byte.
            if next_is_overhead_byte {
                // Keep that that we passed another overhead byte.
                overhead_bytes += 1;
            } else {
                // If it wasn't a overhead byte, we can set this byte to the marker byte.
                output_buffer[i - overhead_bytes] = marker;
            }

            // Check whether the next byte will be a overhead byte.
            next_is_overhead_byte = until_next_marker == 0xff;
        } else {
            // If we are not on a marker or overhead byte we can just copy the value over.
            output_buffer[i - overhead_bytes] = value;
        }

        until_next_marker -= 1;

        if i < INPUT {
            i += 1;
        } else {
            panic!("No marker value found!");
        }
    } + 1;

    (output_buffer, output_buffer_length)
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::ops::Range;

    #[derive(Debug)]
    struct TestVector<const N: usize, const M: usize> {
        unencoded_data: [u8; N],
        encoded_data: [u8; M],
    }

    impl<const N: usize, const M: usize> TestVector<N, M> {
        const fn new(unencoded_data: [u8; N], encoded_data: [u8; M]) -> Self {
            Self {
                unencoded_data,
                encoded_data,
            }
        }

        fn assert_stuff(&self) {
            assert_eq!(stuff::<N, M>(self.unencoded_data, 0x00), self.encoded_data);
        }

        fn assert_unstuff(&self) {
            assert_eq!(
                unstuff::<M, N>(self.encoded_data, 0x00),
                (self.unencoded_data, self.encoded_data.len())
            );
        }

        fn assert_stuff_then_unstuff(&self) {
            assert_eq!(
                unstuff::<M, N>(stuff(self.unencoded_data, 0x00), 0x00),
                (self.unencoded_data, self.encoded_data.len())
            );
        }

        fn assert_unstuff_then_stuff(&self) {
            assert_eq!(
                stuff::<N, M>(unstuff(self.encoded_data, 0x00).0, 0x00),
                self.encoded_data
            );
        }
    }

    fn get_range<const N: usize>(
        mut initial: [u8; N],
        start_index: usize,
        range: Range<u8>,
    ) -> [u8; N] {
        for (index, value) in range.enumerate() {
            initial[index + start_index] = value;
        }

        initial
    }

    const TV_1: TestVector<1, 3> = TestVector::new([0x00], [0x01, 0x01, 0x00]);
    const TV_2: TestVector<2, 4> = TestVector::new([0x00, 0x00], [0x01, 0x01, 0x01, 0x00]);
    const TV_3: TestVector<4, 6> = TestVector::new(
        [0x11, 0x22, 0x00, 0x33],
        [0x03, 0x11, 0x22, 0x02, 0x33, 0x00],
    );
    const TV_4: TestVector<4, 6> = TestVector::new(
        [0x11, 0x22, 0x33, 0x44],
        [0x05, 0x11, 0x22, 0x33, 0x44, 0x00],
    );
    const TV_5: TestVector<4, 6> = TestVector::new(
        [0x11, 0x00, 0x00, 0x00],
        [0x02, 0x11, 0x01, 0x01, 0x01, 0x00],
    );
    fn tv_6() -> TestVector<254, 256> {
        TestVector::new(
            get_range([0; 254], 0, 0x01..0xff),
            get_range(
                {
                    let mut arr = [0; 256];
                    arr[0] = 0xff;
                    arr
                },
                1,
                0x01..0xff,
            ),
        )
    }
    fn tv_7() -> TestVector<255, 257> {
        TestVector::new(
            get_range([0; 255], 0, 0x00..0xff),
            get_range(
                {
                    let mut arr = [0; 257];
                    arr[0] = 0x01;
                    arr[1] = 0xff;
                    arr
                },
                2,
                0x01..0xff,
            ),
        )
    }

    fn tv_8() -> TestVector<255, 258> {
        TestVector::new(
            get_range([0xff; 255], 0, 0x01..0xff),
            get_range(
                {
                    let mut arr = [0; 258];
                    arr[0] = 0xff;
                    arr[255] = 0x02;
                    arr[256] = 0xff;
                    arr
                },
                1,
                0x01..0xff,
            ),
        )
    }

    fn tv_9() -> TestVector<255, 258> {
        TestVector::new(
            get_range(
                {
                    let mut arr = [0xff; 255];
                    arr[254] = 0;
                    arr
                },
                0,
                0x02..0xff,
            ),
            get_range(
                {
                    let mut arr = [0; 258];
                    arr[0] = 0xff;
                    arr[254] = 0xff;
                    arr[255] = 0x01;
                    arr[256] = 0x01;
                    arr
                },
                1,
                0x02..0xff,
            ),
        )
    }

    fn tv_10() -> TestVector<255, 257> {
        TestVector::new(
            get_range(
                {
                    let mut arr = [0xff; 255];
                    arr[253] = 0x00;
                    arr[254] = 0x01;
                    arr
                },
                0,
                0x03..0xff,
            ),
            get_range(
                {
                    let mut arr = [0; 257];
                    arr[0] = 0xfe;
                    arr[253] = 0xff;
                    arr[254] = 0x02;
                    arr[255] = 0x01;
                    arr
                },
                1,
                0x03..0xff,
            ),
        )
    }

    #[test]
    fn stuff_test_vectors() {
        TV_1.assert_stuff();
        TV_2.assert_stuff();
        TV_3.assert_stuff();
        TV_4.assert_stuff();
        TV_5.assert_stuff();
        tv_6().assert_stuff();
        tv_7().assert_stuff();
        tv_8().assert_stuff();
        tv_9().assert_stuff();
        tv_10().assert_stuff();
    }

    #[test]
    fn unstuff_test_vectors() {
        TV_1.assert_unstuff();
        TV_2.assert_unstuff();
        TV_3.assert_unstuff();
        TV_4.assert_unstuff();
        TV_5.assert_unstuff();
        tv_6().assert_unstuff();
        tv_7().assert_unstuff();
        tv_8().assert_unstuff();
        tv_9().assert_unstuff();
        tv_10().assert_unstuff();
    }

    #[test]
    fn inverses() {
        TV_1.assert_stuff_then_unstuff();
        TV_2.assert_stuff_then_unstuff();
        TV_3.assert_stuff_then_unstuff();
        TV_4.assert_stuff_then_unstuff();
        TV_5.assert_stuff_then_unstuff();
        tv_6().assert_stuff_then_unstuff();
        tv_7().assert_stuff_then_unstuff();
        tv_8().assert_stuff_then_unstuff();
        tv_9().assert_stuff_then_unstuff();
        tv_10().assert_stuff_then_unstuff();

        TV_1.assert_unstuff_then_stuff();
        TV_2.assert_unstuff_then_stuff();
        TV_3.assert_unstuff_then_stuff();
        TV_4.assert_unstuff_then_stuff();
        TV_5.assert_unstuff_then_stuff();
        tv_6().assert_unstuff_then_stuff();
        tv_7().assert_unstuff_then_stuff();
        tv_8().assert_unstuff_then_stuff();
        tv_9().assert_unstuff_then_stuff();
        tv_10().assert_unstuff_then_stuff();
    }
}
