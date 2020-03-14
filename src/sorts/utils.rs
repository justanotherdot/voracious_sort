use rayon::prelude::*;

use std::sync::mpsc::channel;

use super::super::{RadixKey, Radixable};

#[derive(Copy, Clone, Debug, Default)]
pub struct Params {
    pub level: usize,
    pub radix: usize,
    pub offset: usize,
    pub max_level: usize,
    pub radix_range: usize,
}

impl Params {
    pub fn new(
        level: usize,
        radix: usize,
        offset: usize,
        max_level: usize,
    ) -> Params {
        Params {
            level,
            radix,
            offset,
            max_level,
            radix_range: (2 as usize).pow(radix as u32),
        }
    }

    pub fn new_level(&self, level: usize) -> Params {
        Params { level, ..(*self) }
    }
}

#[inline]
pub fn swap_range<T>(a: &mut [T], len: usize, i: usize, j: usize) {
    unsafe {
        let pa: *mut T = a.get_unchecked_mut(i);
        let pb: *mut T = a.get_unchecked_mut(j);
        std::ptr::swap_nonoverlapping(pa, pb, len);
    }
}

struct SafePtr<T: ?Sized>(*mut T);
unsafe impl<T: ?Sized> Send for SafePtr<T> {}
unsafe impl<T: ?Sized> Sync for SafePtr<T> {}

#[inline]
pub fn swap_range_mt<T: Radixable<K>, K: RadixKey>(arr: &mut [T], len: usize, i: usize, j: usize) {
    let ptr = SafePtr(arr.as_mut_ptr());

    let indices = vec![0; len];

    indices
        .par_iter()
        .enumerate()
        .for_each(|(offset, _)| unsafe {
            let SafePtr(ptr) = ptr;
            let p1: *mut _ = ptr.add(i + offset);
            let p2: *mut _ = ptr.add(j + offset);
            std::ptr::swap(p1, p2);
        });
}

pub fn perform_swaps<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    swaps: Vec<(usize, usize, usize)>,
    offset: usize,
) {
    for (len, i1, i2) in swaps.iter() {
        swap_range(arr, *len, *i1 - offset, *i2 - offset);
    }
}

pub fn perform_swaps_mt<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    swaps: Vec<(usize, usize, usize)>,
    offset: usize,
) {
    let ptr = SafePtr(arr.as_mut_ptr());

    swaps
        .par_iter()
        .for_each(|(len, i, j)| unsafe {
            let SafePtr(ptr) = ptr;
            let p1: *mut _ = ptr.add(i - offset);
            let p2: *mut _ = ptr.add(j - offset);
            std::ptr::swap_nonoverlapping(p1, p2, *len);
        });
}

#[inline]
pub fn copy_nonoverlapping<T>(
    source: &mut [T],
    destination: &mut [T],
    length: usize,
) {
    unsafe {
        std::ptr::copy_nonoverlapping(
            source.as_ptr(),
            destination.get_unchecked_mut(0),
            length,
        );
    }
}

pub fn prefix_sums(
    histogram: &[usize],
) -> (Vec<usize>, Vec<usize>, Vec<usize>) {
    let p_sums = histogram.iter().fold(vec![0], |mut acc, count| {
        acc.push(*count as usize + acc.last().unwrap());
        acc
    });

    let mut heads = p_sums.to_vec();
    heads.split_off(p_sums.len() - 1);
    let tails = p_sums.to_vec().split_off(1);

    (p_sums, heads, tails)
}

pub fn only_one_bucket_filled(histogram: &[usize]) -> bool {
    let mut count = 0;

    for item in histogram {
        if *item > 0 {
            count += 1;
        }

        if count > 1 {
            return false;
        }
    }

    true
}

pub fn split_into_chunks<T, K>(arr: &mut [T], chunk_n: usize) -> Vec<&mut [T]>
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    let part_size = arr.len() / chunk_n;

    let mut parts = Vec::new();
    let mut rest = arr;
    for _ in 0..(chunk_n - 1) {
        let (fst, snd) = rest.split_at_mut(part_size);
        rest = snd;
        parts.push(fst);
    }
    if rest.len() > 0 {
        parts.push(rest);
    }

    parts
}

pub fn offset_from_bits<T, K>(
    _arr: &mut [T],
    biggest: <<T as Radixable<K>>::Key as RadixKey>::Key,
    radix: usize,
    bits: usize,
    zero: <<T as Radixable<K>>::Key as RadixKey>::Key,
    one: <<T as Radixable<K>>::Key as RadixKey>::Key,
) -> (usize, usize)
where
    T: Radixable<K>,
    K: RadixKey,
{
    let mut count = 0;
    let mut buf = biggest;

    while buf != zero {
        buf = buf >> one;
        count += 1;
    }

    let offset = if count % radix == 0 {
        bits - count
    } else {
        let q = count / radix;
        let total_bits = (q + 1) * radix;

        if total_bits > bits {
            0
        } else {
            bits - total_bits
        }
    };

    (offset, bits - count)
}

pub fn compute_offset<T: Radixable<K> + Copy, K: RadixKey>(
    arr: &mut [T],
    radix: usize,
) -> (usize, usize) {
    let dummy = arr[0];
    let max = arr.iter().map(|item| item.into_key_type()).max().unwrap();

    offset_from_bits(
        arr,
        max,
        radix,
        dummy.type_size(),
        dummy.default_key(),
        dummy.one(),
    )
}

pub fn compute_max_level(bits: usize, offset: usize, radix: usize) -> usize {
    let rest = bits - offset;
    if rest % radix != 0 {
        (rest / radix) + 1
    } else {
        rest / radix
    }
}

pub fn aggregate_histograms(histograms: &Vec<Vec<usize>>) -> Vec<usize> {
    let mut global_histogram = vec![0; histograms[0].len()];

    histograms.iter().for_each(|histogram| {
        histogram.iter().enumerate().for_each(|(i, v)| {
            global_histogram[i] += v;
        });
    });

    global_histogram
}

pub fn get_empty_histograms(p: &Params, partial: usize) -> Vec<Vec<usize>> {
    let mut histograms = Vec::new();

    for _ in 0..partial {
        let histogram = vec![0; p.radix_range];
        histograms.push(histogram);
    }

    histograms
}

pub fn get_histogram<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
    mask: <<T as Radixable<K>>::Key as RadixKey>::Key,
    shift: usize,
) -> Vec<usize> {
    let mut histogram = vec![0; p.radix_range];

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;

    for q in 0..quotient {
        let i = q * 4;
        unsafe {
            let bucket0 = arr.get_unchecked(i).extract(mask, shift);
            let bucket1 = arr.get_unchecked(i + 1).extract(mask, shift);
            let bucket2 = arr.get_unchecked(i + 2).extract(mask, shift);
            let bucket3 = arr.get_unchecked(i + 3).extract(mask, shift);
            histogram[bucket0] += 1;
            histogram[bucket1] += 1;
            histogram[bucket2] += 1;
            histogram[bucket3] += 1;
        }
    }

    let offset = quotient * 4;
    for i in 0..remainder {
        unsafe {
            let bucket = arr.get_unchecked(offset + i).extract(mask, shift);
            histogram[bucket] += 1;
        }
    }

    histogram
}

pub fn get_histogram_mt<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
    mask: <<T as Radixable<K>>::Key as RadixKey>::Key,
    shift: usize,
    pool: &rayon::ThreadPool,
    chunk_n: usize,
) -> Vec<usize> {
    let parts = split_into_chunks(arr, chunk_n);
    let mut histograms: Vec<Vec<usize>> = Vec::new();
    let mut receivers = Vec::new();

    pool.scope(|s| {
        for part in parts.into_iter() {
            let (sender, receiver) = channel();
            receivers.push(receiver);
            s.spawn(move|_| {
                let h = get_histogram(part, p, mask, shift);
                sender.send(h).unwrap();
            });
        }
    });

    for receiver in receivers.iter() {
        histograms.push(receiver.recv().unwrap());
    }

    aggregate_histograms(&histograms)
}

pub fn _get_full_histogram<T, K>(arr: &mut [T], p: &Params) -> Vec<Vec<usize>>
where
    T: Radixable<K>,
    K: RadixKey,
{
    let dummy = arr[0];
    let mut histograms = Vec::new();
    for level in 0..p.max_level {
        let mut histogram = vec![0; p.radix_range];
        let (mask, shift) = dummy.get_mask_and_shift(&p.new_level(level));

        arr.iter().for_each(|element| {
            histogram[element.extract(mask, shift)] += 1;
        });

        histograms.push(histogram);
    }

    histograms
}

pub fn get_full_histogram_except_for_last_level<T, K>(
    arr: &mut [T],
    p: &Params,
) -> Vec<Vec<usize>>
where
    T: Radixable<K> + Copy + PartialOrd,
    K: RadixKey,
{
    let dummy = arr[0];
    let mut histograms = Vec::new();
    for level in 0..(p.max_level - 1) {
        let mut histogram = vec![0; p.radix_range];
        let (mask, shift) = dummy.get_mask_and_shift(&p.new_level(level));

        arr.iter().for_each(|element| {
            histogram[element.extract(mask, shift)] += 1;
        });

        histograms.push(histogram);
    }

    histograms
}

pub fn get_next_two_histograms<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
) -> Vec<Vec<usize>> {
    let dummy = arr[0];
    let shift = dummy.usize_to_keytype(p.radix);
    let (_, fst_shift) = dummy.get_mask_and_shift_from_left(&p);
    let fst_shift = dummy.usize_to_keytype(fst_shift - p.radix);

    let mut histograms = get_empty_histograms(p, 2);
    let default_mask = dummy.default_mask(p.radix);

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;
    let offset = quotient * 4;

    for q in 0..quotient {
        unsafe {
            let i = q * 4;
            let mut v0 = arr.get_unchecked(i).into_key_type();
            let mut v1 = arr.get_unchecked(i + 1).into_key_type();
            let mut v2 = arr.get_unchecked(i + 2).into_key_type();
            let mut v3 = arr.get_unchecked(i + 3).into_key_type();
            v0 = v0 >> fst_shift;
            v1 = v1 >> fst_shift;
            v2 = v2 >> fst_shift;
            v3 = v3 >> fst_shift;
            histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            v0 = v0 >> shift;
            v1 = v1 >> shift;
            v2 = v2 >> shift;
            v3 = v3 >> shift;
            histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
            histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
        }
    }
    for i in 0..remainder {
        unsafe {
            let mut v = arr.get_unchecked(offset + i).into_key_type();
            v = v >> fst_shift;
            histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
            v = v >> shift;
            histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
        }
    }

    histograms
}

pub fn get_partial_histograms_fast<T: Radixable<K>, K: RadixKey>(
    arr: &mut [T],
    p: &Params,
    partial: usize,
) -> Vec<Vec<usize>> {
    if partial > 5 {
        panic!("[RadixableForContainer] Array size can't be that huge !");
    }

    let dummy = arr[0];
    let mut histograms = get_empty_histograms(p, partial);
    let default_mask = dummy.default_mask(p.radix);
    let shift = dummy.usize_to_keytype(p.radix);
    let bits = dummy.type_size();
    let fs = if p.radix * partial > bits - p.offset {
        0
    } else {
        bits - p.offset - p.radix * partial
    };
    let fs = dummy.usize_to_keytype(fs);

    if partial == 1 {
        for element in arr.iter() {
            let value = element.into_key_type() >> fs;

            histograms[0][dummy.keytype_to_usize(value & default_mask)] += 1;
        }
    } else if partial == 2 {
        for element in arr.iter() {
            let mut value = element.into_key_type() >> fs;

            histograms[1][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][dummy.keytype_to_usize(value & default_mask)] += 1;
        }
    } else if partial == 3 {
        for element in arr.iter() {
            let mut value = element.into_key_type() >> fs;

            histograms[2][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[1][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][dummy.keytype_to_usize(value & default_mask)] += 1;
        }
    } else if partial == 4 {
        for element in arr.iter() {
            let mut value = element.into_key_type() >> fs;

            histograms[3][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[2][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[1][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][dummy.keytype_to_usize(value & default_mask)] += 1;
        }
    } else if partial == 5 {
        for element in arr.iter() {
            let mut value = element.into_key_type() >> fs;

            histograms[4][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[3][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[2][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[1][dummy.keytype_to_usize(value & default_mask)] += 1;
            value = value >> shift;

            histograms[0][dummy.keytype_to_usize(value & default_mask)] += 1;
        }
    }

    histograms
}

pub fn get_full_histograms_fast<T, K>(arr: &mut [T], p: &Params) -> Vec<Vec<usize>>
where
    T: Radixable<K>,
    K: RadixKey,
{
    let dummy = arr[0];
    let mut histograms = get_empty_histograms(p, p.max_level);
    let default_mask = dummy.default_mask(p.radix);
    let shift = dummy.usize_to_keytype(p.radix);

    let quotient = arr.len() / 4;
    let remainder = arr.len() % 4;
    let offset = quotient * 4;

    if p.max_level == 1 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let v0 = arr.get_unchecked(i).into_key_type();
                let v1 = arr.get_unchecked(i + 1).into_key_type();
                let v2 = arr.get_unchecked(i + 2).into_key_type();
                let v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let v = arr.get_unchecked(offset + i).into_key_type();
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 2 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.get_unchecked(i).into_key_type();
                let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                let mut v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.get_unchecked(offset + i).into_key_type();
                histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 3 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.get_unchecked(i).into_key_type();
                let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                let mut v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.get_unchecked(offset + i).into_key_type();
                histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 4 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.get_unchecked(i).into_key_type();
                let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                let mut v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.get_unchecked(offset + i).into_key_type();
                histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 5 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.get_unchecked(i).into_key_type();
                let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                let mut v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.get_unchecked(offset + i).into_key_type();
                histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 6 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.get_unchecked(i).into_key_type();
                let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                let mut v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.get_unchecked(offset + i).into_key_type();
                histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 7 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.get_unchecked(i).into_key_type();
                let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                let mut v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.get_unchecked(offset + i).into_key_type();
                histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    } else if p.max_level == 8 {
        for q in 0..quotient {
            unsafe {
                let i = q * 4;
                let mut v0 = arr.get_unchecked(i).into_key_type();
                let mut v1 = arr.get_unchecked(i + 1).into_key_type();
                let mut v2 = arr.get_unchecked(i + 2).into_key_type();
                let mut v3 = arr.get_unchecked(i + 3).into_key_type();
                histograms[7][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[7][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[7][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[7][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[6][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[6][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[6][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[6][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[5][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[5][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[4][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[4][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[3][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[3][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[2][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[2][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[1][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[1][dummy.keytype_to_usize(v3 & default_mask)] += 1;
                v0 = v0 >> shift;
                v1 = v1 >> shift;
                v2 = v2 >> shift;
                v3 = v3 >> shift;
                histograms[0][dummy.keytype_to_usize(v0 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v1 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v2 & default_mask)] += 1;
                histograms[0][dummy.keytype_to_usize(v3 & default_mask)] += 1;
            }
        }
        for i in 0..remainder {
            unsafe {
                let mut v = arr.get_unchecked(offset + i).into_key_type();
                histograms[7][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[6][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[5][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[4][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[3][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[2][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[1][dummy.keytype_to_usize(v & default_mask)] += 1;
                v = v >> shift;
                histograms[0][dummy.keytype_to_usize(v & default_mask)] += 1;
            }
        }
    }

    histograms
}
