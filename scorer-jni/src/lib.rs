#![feature(test)]

use jni::JNIEnv;
use jni::objects::JClass;
use jni::sys::{jfloat, jint, jobject};

use packed_simd::{f32x8, m32x8};

use std::slice;
use std::cmp::{min, max};

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
    let no_adv_scores = f32x8::splat(-1.0);
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

        let new_scores_vec = ScoreDataSimd {
            is_prosale_view_only: prosale_view_only_vec,

            scores: scores_vec,
            min_scores,
            max_score,

            adv_weights: adv_weights_vec,
            min_adv_weights,
            no_adv_scores,
            min_adv_boosts,
            max_adv_boosts,

            slope,
            intercept,
        }.calc_scores();

//        println!("!!!!");
        unsafe {
            new_scores_vec.write_to_slice_aligned_unchecked(
                slice::from_raw_parts_mut(scores_ptr as *mut f32, 8)
            );
        }
    }
}

struct ScoreDataSimd {
    is_prosale_view_only: m32x8,

    scores: f32x8,
    min_scores: f32x8,
    max_score: f32,

    adv_weights: f32x8,
    min_adv_weights: f32x8,
    no_adv_scores: f32x8,
    min_adv_boosts: f32x8,
    max_adv_boosts: f32x8,

    slope: f32,
    intercept: f32,
}

impl ScoreDataSimd {
    // #[inline(always)]
    fn calc_scores(&self) -> f32x8 {
        let is_adv = self.adv_weights.gt(self.min_adv_weights) & self.scores.ge(self.min_scores);
        self.is_prosale_view_only.select(
            is_adv.select(
                self.max_score * self.max_adv_boosts.min(
                    self.min_adv_boosts.max(
                        self.adv_weights * self.slope + self.intercept
                    )
                ),
                self.no_adv_scores
            ),
            self.scores
        )
    }
}

struct ScoreData {
    is_prosale_view_only: bool,

    score: f32,
    min_score: f32,
    max_score: f32,

    adv_weight: f32,
    min_adv_weight: f32,
    no_adv_score: f32,
    min_adv_boost: f32,
    max_adv_boost: f32,

    slope: f32,
    intercept: f32,
}

impl ScoreData {
    fn calc_score(&self) -> f32 {
        if !self.is_prosale_view_only {
            return self.score;
        }
        if self.adv_weight < self.min_adv_weight || self.score < self.min_score {
            return self.no_adv_score;
        }
        self.max_score * self.max_adv_boost.min(
            self.min_adv_boost.max(
                self.adv_weight * self.slope + self.intercept
            )
        )
    }
}

#[cfg(test)]
mod tests {
    use packed_simd::{f32x8, m32x8};

    use std::ops::*;
    use std::slice;

    extern crate test;
    use test::{Bencher, black_box};

    use super::{ScoreData};
    use crate::ScoreDataSimd;

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

    struct TestData {
        scores: AlignedArray<f32>,
        adv_weights: AlignedArray<f32>,
        prosale_view_only: AlignedArray<bool>,
        prosale_view_only_masks: AlignedArray<u32>,
        res: AlignedArray<f32>,
    }
    impl TestData {
        fn gen() -> TestData {
            let mut scores = AlignedArray([f32::default(); BATCH_SIZE]);
            let mut adv_weights = AlignedArray([f32::default(); BATCH_SIZE]);
            let mut prosale_view_only = AlignedArray([bool::default(); BATCH_SIZE]);
            let mut prosale_view_only_masks = AlignedArray([u32::default(); BATCH_SIZE]);
            let mut res = AlignedArray([f32::default(); BATCH_SIZE]);
            for i in 0..BATCH_SIZE {
                scores.0[i] = i as f32;
                adv_weights.0[i] = 1.0 / scores.0[i] + 1.0;
                prosale_view_only.0[i] = i % 2 == 0;
                prosale_view_only_masks.0[i] = (i % 2) as u32;
            }
            TestData {
                scores,
                adv_weights,
                prosale_view_only,
                prosale_view_only_masks,
                res,
            }
        }
    }

    #[bench]
    fn benchmark_simple(b: &mut Bencher) {
        let mut data: TestData = TestData::gen();
        b.iter(|| {
            for i in 0..BATCH_SIZE {
                let score = ScoreData {
                    is_prosale_view_only: data.prosale_view_only.0[i],

                    score: data.scores.0[i],
                    min_score: 1.0,
                    max_score: 10.0,

                    adv_weight: data.scores.0[i],
                    min_adv_weight: 0.1,
                    no_adv_score: -1.0,
                    min_adv_boost: 2.0,
                    max_adv_boost: 4.0,

                    slope: 0.25,
                    intercept: 0.5,

                }.calc_score();
                data.res.0[i] = score;
            }
            black_box(&data);
        });
    }

    #[bench]
    fn benchmark_simd(b: &mut Bencher) {
        let mut data: TestData = TestData::gen();

        b.iter(|| {

            let min_scores = f32x8::splat(1.0);
            let min_adv_weights = f32x8::splat(0.1);
            let no_adv_scores = f32x8::splat(-1.0);
            let min_adv_boosts = f32x8::splat(2.0);
            let max_adv_boosts = f32x8::splat(4.0);
            let prosale_view_only_masks_ptr = data.prosale_view_only_masks.0.as_ptr() as *const m32x8;
            let scores_ptr = data.scores.0.as_ptr() as *const f32x8;
            let adv_weights_ptr = data.adv_weights.0.as_ptr() as *const f32x8;
            let res_ptr = data.res.0.as_ptr() as *const f32x8;

            let mut i = 0;
            while i < BATCH_SIZE / 8 {
                let scores = ScoreDataSimd {
                    is_prosale_view_only: unsafe {
                        *(prosale_view_only_masks_ptr.offset(i as isize))
                    },

                    scores: unsafe {
                        *(scores_ptr.offset(i as isize))
                    },
                    min_scores,
                    max_score: 10.0,

                    adv_weights: unsafe {
                        *(adv_weights_ptr.offset(i as isize))
                    },
                    min_adv_weights,
                    no_adv_scores,
                    min_adv_boosts,
                    max_adv_boosts,

                    slope: 0.25,
                    intercept: 0.5,
                }.calc_scores();

                // scores.write_to_slice_aligned(&mut data.res.0[i*8..]);
                unsafe {
                    scores.write_to_slice_aligned_unchecked(
                        slice::from_raw_parts_mut(res_ptr as *mut f32, 8)
                    );
                }

                i += 1;
            }
            black_box(&data);
        });
    }
}
