//! A no_std Consistent Overhead Byte Stuffing library

#![no_std]
#![warn(missing_docs)]

use core::convert::TryInto;

struct MarkerInfo {
    index: usize,
    points_to: usize,
}

impl MarkerInfo {
    fn new(index: usize, points_to: usize) -> MarkerInfo {
        MarkerInfo { index, points_to }
    }

    fn does_it_point_to(&self, out_buffer_index: usize) -> bool {
        self.points_to == out_buffer_index
    }

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

/// Stuffs an input buffer into a output buffer
pub fn stuff<const INPUT: usize, const OUTPUT: usize>(
    buff: [u8; INPUT],
    marker: u8,
) -> [u8; OUTPUT] {
    let mut output_buffer: [u8; OUTPUT] = [0; OUTPUT];

    // Keep track of where the last marker was.
    // This always has one in the beginning, which is the overhead byte.
    let mut last_marker = MarkerInfo::new(0, 0xff);

    // Every time we set additional overhead marker, we should increase the offset.
    // This way we keep track what the relationship is between the input array indices and the
    // output array indices.
    let mut overhead_bytes = 1;

    // Loop through all the input bytes.
    for i in 0..INPUT {
        // Fetch the value of the input byte array.
        let value = buff[i];

        if last_marker.does_it_point_to(overhead_bytes + i) {
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
    last_marker.adjust_accordingly(&mut output_buffer, INPUT + overhead_bytes);
    // And we set the value to the marker value in the output buffer.
    output_buffer[INPUT + overhead_bytes] = marker;

    output_buffer
}

/// Unstuffs a input buffer into a output buffer
pub fn unstuff<const INPUT: usize, const OUTPUT: usize>(
    buff: [u8; INPUT],
    marker: u8,
) -> [u8; OUTPUT] {
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
    for i in 1..INPUT {
        // Fetch the value from the input buffer.
        let value = buff[i];

        // If we value is the marker, we know we have reached the end.
        if value == marker {
            break;
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
    }

    output_buffer
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
                self.unencoded_data
            );
        }

        fn assert_stuff_then_unstuff(&self) {
            assert_eq!(
                unstuff::<M, N>(stuff(self.unencoded_data, 0x00), 0x00),
                self.unencoded_data
            );
        }

        fn assert_unstuff_then_stuff(&self) {
            assert_eq!(
                stuff::<N, M>(unstuff(self.encoded_data, 0x00), 0x00),
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
