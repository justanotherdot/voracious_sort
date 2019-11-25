use super::super::algo::k_way_merge::k_way_merge;
use super::super::algo::verge_sort_heuristic::verge_sort_preprocessing;
use super::super::{Radixable, RadixableForContainer};
use super::counting_sort::counting_sort;
use super::msd_sort::copy_by_histogram;
use super::utils::{
    copy_nonoverlapping, get_full_histograms_fast, only_one_bucket_filled,
    prefix_sums, Params,
};

pub fn lsd_radixsort_body<T>(arr: &mut [T], p: Params)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer<T = T>,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let size = arr.len();
    let mut buffer: Vec<T> = vec![arr[0]; size];
    let mut index = 0;

    let histograms = get_full_histograms_fast(arr, &p);

    let mut t1 = arr;
    let t2 = &mut buffer;
    let mut t2 = t2.as_mut_slice();

    for level in (p.level..p.max_level).rev() {
        if only_one_bucket_filled(&histograms[level]) {
            continue;
        }

        let (mut source, mut destination) =
            if index == 0 { (t1, t2) } else { (t2, t1) };
        let (mask, shift) = source.get_mask_and_shift(&p.new_level(level));
        let (_, mut heads, _) = prefix_sums(&histograms[level]);

        copy_by_histogram(
            source.len(),
            &mut source,
            &mut destination,
            &mut heads,
            mask,
            shift,
        );

        index = 1 - index;

        if index == 1 {
            t1 = source;
            t2 = destination;
        } else {
            t2 = source;
            t1 = destination;
        }
    }

    if index == 1 {
        copy_nonoverlapping(t2, t1, size);
    }
}

pub fn lsd_radixsort_aux<T>(
    arr: &mut [T],
    radix: usize,
    heuristic: bool,
    min_cs2: usize,
) where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer<T = T>,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let (offset, _) = arr.compute_offset(radix);
    let max_level = arr.compute_max_level(offset, radix);
    let params = Params::new(0, radix, offset, max_level);

    if heuristic {
        if max_level == 1 {
            counting_sort(arr, 8);
        } else if max_level == 2 && arr.len() >= min_cs2 {
            counting_sort(arr, 16);
        } else {
            lsd_radixsort_body(arr, params);
        }
    } else {
        lsd_radixsort_body(arr, params);
    }
}

pub fn lsd_radixsort<T>(arr: &mut [T], radix: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer<T = T>,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, radix| {
        lsd_radixsort_aux(arr, radix, false, 0)
    });
    k_way_merge(arr, &mut separators);
}

pub fn lsd_radixsort_heu<T>(arr: &mut [T], radix: usize, min_cs2: usize)
where
    T: Radixable<KeyType = <[T] as RadixableForContainer>::KeyType>
        + Copy
        + PartialOrd,
    [T]: RadixableForContainer<T = T>,
{
    if arr.len() <= 128 {
        arr.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        return;
    }

    let mut separators = verge_sort_preprocessing(arr, radix, &|arr, radix| {
        lsd_radixsort_aux(arr, radix, true, min_cs2)
    });
    k_way_merge(arr, &mut separators);
}