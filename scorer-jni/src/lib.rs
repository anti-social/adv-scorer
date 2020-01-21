#![feature(test)]

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jfloat, jint, jobject};

use packed_simd::{f32x8, m32x8};

use std::slice;

#[no_mangle]
pub extern "system" fn Java_dev_evo_advscorer_AdvScorerJni_calcScores(
    env: JNIEnv,
    _class: JClass,
    size: jint,
    scores: jobject, // in-out argument
    adv_weights: jobject,
    prosale_view_only: jobject,
    min_score: jfloat,
    max_score: jfloat,
    min_adv_boost: jfloat,
    max_adv_boost: jfloat,
    slope: jfloat,
    intercept: jfloat,
) {
//    println!("Size: {}", size);
    assert!(size > 0);
    assert!(size % 8 == 0);
    let num_chunks = size as usize / 8;

    let scores = env.get_direct_buffer_address(scores.into()).unwrap();
    let scores_ptr = scores.as_ptr() as *const f32x8;
//    println!("Scores buf len: {}", scores.len());
//    println!("Scores ptr: {:?}", scores_ptr);
    assert!(scores.len() / 32 >= num_chunks);

    let adv_weights = env.get_direct_buffer_address(adv_weights.into()).unwrap();
    let adv_weights_ptr = adv_weights.as_ptr() as *const f32x8;
//    println!("Weights buf len: {}", adv_weights.len());
//    println!("Weights ptr: {:?}", adv_weights_ptr);
    assert!(adv_weights.len() / 32 >= num_chunks);

    let prosale_view_only = env.get_direct_buffer_address(prosale_view_only.into()).unwrap();
    let prosale_view_only_ptr = prosale_view_only.as_ptr() as *const m32x8;
//    println!("Prosale view only buf len: {}", prosale_view_only.len());
//    println!("Prosale view only ptr: {:?}", prosale_view_only_ptr);
    assert!(prosale_view_only.len() >= num_chunks);

    let min_scores = f32x8::splat(min_score);
    let min_adv_weights = f32x8::splat(0.0);
    let no_adv_score = f32x8::splat(-1.0);
    let min_adv_boosts = f32x8::splat(min_adv_boost);
    let max_adv_boosts = f32x8::splat(max_adv_boost);

    for i in 0..num_chunks {
//        println!("{}", i);
        let scores_vec = unsafe {
            *scores_ptr.offset(i as isize)
        };
//        println!("!");
        let adv_weights_vec = unsafe {
            *adv_weights_ptr.offset(i as isize)
        };
//        println!("!!");
        let prosale_view_only_vec = unsafe {
            *prosale_view_only_ptr.offset(i as isize)
        };
//        println!("!!!");

        let new_scores_vec = calc_scores(
            scores_vec,
            min_scores,
            max_score,
            adv_weights_vec,
            min_adv_weights,
            prosale_view_only_vec,
            no_adv_score,
            slope,
            intercept,
            min_adv_boosts,
            max_adv_boosts,
        );

//        println!("!!!!");
        unsafe {
            new_scores_vec.write_to_slice_aligned_unchecked(
                slice::from_raw_parts_mut(scores_ptr as *mut f32, 8)
            );
        }
    }
}

// #[inline(always)]
fn calc_scores(
    scores: f32x8,
    min_score: f32x8,
    max_score: f32,
    adv_weights: f32x8,
    min_adv_weight: f32x8,
    is_prosale_view_only: m32x8,
    no_adv_score: f32x8,
    slope: f32,
    intercept: f32,
    min_adv_boost: f32x8,
    max_adv_boost: f32x8,
) -> f32x8 {

    let is_adv = adv_weights.gt(min_adv_weight) & scores.ge(min_score);
    is_prosale_view_only.select(
        is_adv.select(
            max_score * max_adv_boost.min(
                min_adv_boost.max(
                    adv_weights * slope + intercept
                )
            ),
            no_adv_score
        ),
        scores
    )
}

#[cfg(test)]
mod tests {
    use packed_simd::{f32x8, m32x8};

    extern crate test;
    use test::{Bencher, black_box};

    use super::calc_scores;

//    #[test]
//    fn test_calc_scores() {
//        let scores = calc_scores(
//            f32x8::new(0.1, 0.5, 1.0, 1.1, 1.2, 1.3, 1.4, 1.5),
//            f32x8::splat(1.0),
//            100.0,
//            f32x8::new(-0.1, 0.0, 0.1, 1.0, 2.0, 3.0, 10.0, 100.0),
//            f32x8::splat(0.0),
//            m32x8::splat(true), // TODO
//            0.25,
//            0.5,
//            f32x8::splat(1.0),
//            f32x8::splat(10.0),
//        );
//        assert_eq!(
//            scores,
//            f32x8::new(0.1, 0.5, 1.0, 1.0, 1.0, 1.25, 3.0, 10.0)
//        );
//    }

    const BATCH_SIZE: usize = 1024 * 64;
    const ALIGN: usize = 32;

    #[repr(C, align(32))]
    struct AlignedArray<T>([T; BATCH_SIZE]);

    fn offset_for_align(addr: usize, align: usize) -> usize {
        let exceed = addr % align;
        if exceed == 0 {
            return 0;
        }
        return align - exceed;
    }

    #[bench]
    fn benchmark_safe(b: &mut Bencher) {
        let mut scores = AlignedArray([0.0; BATCH_SIZE]);
        let mut adv_weights = AlignedArray([0.0; BATCH_SIZE]);
        let mut prosale_only_view = Vec::with_capacity(BATCH_SIZE + ALIGN);
        let mut res = AlignedArray([0.0; BATCH_SIZE]);
        for i in 0..BATCH_SIZE {
            scores.0[i] = i as f32;
            adv_weights.0[i] = 1.0 / scores.0[i] + 1.0;
            prosale_only_view.push(false);
        }

        b.iter(|| {
            let mut i = 0;
            while i < BATCH_SIZE {
                let new_scores = calc_scores(
                    f32x8::from_slice_aligned(&scores.0[i..]),
                    f32x8::splat(1.0),
                    100.0,
                    f32x8::from_slice_aligned(&adv_weights.0[i..]),
                    f32x8::splat(0.0),
                    m32x8::splat(true), // TODO
                    f32x8::splat(-1.0),
                    0.25,
                    0.5,
                    f32x8::splat(1.0),
                    f32x8::splat(10.0),
                );
                new_scores.write_to_slice_aligned(&mut res.0);

                i += 8;
            }
            black_box(&res);
        });
    }
}
