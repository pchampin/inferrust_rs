use rayon::prelude::*;
use std::cell::Ref;
use std::cell::RefCell;
use std::mem;

use super::NodeDictionary;

pub struct TripleStore {
    pub elem: Vec<Chunk>,
    size: usize,
}

#[derive(PartialEq)]
pub struct Chunk {
    so: Vec<[u64; 2]>,
    os: Vec<[u64; 2]>,
    os_sorted: bool,
}

unsafe impl Send for Chunk {}

impl Chunk {
    pub fn so_sort(
        &mut self,
        hist: &mut Vec<usize>,
        hist2: &mut Vec<usize>,
        cumul: &mut Vec<usize>,
        min: u64,
        max: u64,
        width: usize,
    ) -> usize {
        self.os_sorted = false;
        bucket_sort_pairs(&mut self.so, hist, hist2, cumul, min, max, width)
    }

    unsafe fn os_sort(&self) {
        // TODO
        if !self.os.is_empty() {
            let (min, max) = self
                .os
                .iter()
                .flat_map(|pair| pair.iter())
                .fold((u64::max_value(), 0), |acc, &x| {
                    (acc.0.min(x), acc.1.max(x))
                });
            let width = (max - min + 1) as usize;
            let mut hist: Vec<usize> = vec![0; width];
            let mut hist2: Vec<usize> = Vec::with_capacity(width);
            let mut cumul: Vec<usize> = vec![0; width];
            let content = &self.os as *const Vec<[u64; 2]> as *mut Vec<[u64; 2]>;
            let mut content = content.as_mut().unwrap();
            // content.foo();
            bucket_sort_pairs(
                &mut content,
                &mut hist,
                &mut hist2,
                &mut cumul,
                min,
                max,
                width,
            );
        }
        *(&self.os_sorted as *const bool as *mut bool)
            .as_mut()
            .unwrap() = true;
    }

    pub fn so(&self) -> &Vec<[u64; 2]> {
        &self.so
    }

    pub fn os(&self) -> &Vec<[u64; 2]> {
        if !self.os_sorted {
            unsafe {
                self.os_sort();
            }
        }
        &self.os
    }

    fn res_to_prop(&mut self, res: u64, prop: u64) {
        for pair in self.so.iter_mut() {
            for i in 0..=1 {
                if pair[i] == res {
                    pair[i] = prop;
                }
            }
        }
        for pair in self.os.iter_mut() {
            for i in 0..=1 {
                if pair[i] == res {
                    pair[i] = prop;
                }
            }
        }
    }

    pub fn add_so(&mut self, so: [u64; 2]) {
        self.so.push(so);
    }

    pub fn add_os(&mut self, os: [u64; 2]) {
        self.os.push(os);
    }
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            so: Vec::new(),
            os: Vec::new(),
            os_sorted: false,
        }
    }
}

impl TripleStore {
    pub fn new() -> Self {
        let elem = Vec::new();
        Self { elem, size: 0 }
    }

    pub fn add_triple(&mut self, triple: [u64; 3]) {
        let [is, ip, io] = triple;
        let ip_to_store = NodeDictionary::prop_idx_to_idx(ip);
        // dbg!(is, ip, io);
        // dbg!(is, ip_to_store, io);
        if ip_to_store >= self.elem.len() {
            self.elem.resize_with(ip_to_store + 1, Default::default);
        }
        self.add_triple_raw(is, ip_to_store, io);
    }

    pub fn add_all(&mut self, other: Self) {
        if other.elem.len() > self.elem.len() {
            self.elem.resize_with(other.elem.len(), Default::default);
        }
        for ip in 0..other.elem.len() {
            for [is, io] in other.elem[ip].so() {
                self.add_triple_raw(*is, ip, *io);
            }
        }
    }

    /// # Pre-condition
    /// `self.elem` must have an element at index `ip`
    #[inline]
    pub fn add_triple_raw(&mut self, is: u64, ip: usize, io: u64) {
        self.elem[ip].add_so([is, io]);
        self.elem[ip].add_os([io, is]);
        self.size += 1;
    }

    pub fn sort(&mut self) {
        let (min, max, width) = self.width();
        self.size = self
            .elem
            .par_iter_mut()
            .map_init(
                || {
                    let hist: Vec<usize> = vec![0; width];
                    let hist2: Vec<usize> = Vec::with_capacity(width);
                    let cumul: Vec<usize> = vec![0; width];
                    (hist, hist2, cumul)
                },
                |(hist, hist2, cumul), chunk| chunk.so_sort(hist, hist2, cumul, min, max, width),
            )
            .sum();
    }

    pub fn res_to_prop(&mut self, res: u64, prop: u32) {
        for chunk in &mut self.elem {
            chunk.res_to_prop(res, prop.into());
        }
        /////////
    }

    pub fn size(&mut self) -> usize {
        self.size
    }

    pub fn width(&mut self) -> (u64, u64, usize) {
        let (min, max) = self
            .elem
            .iter()
            .map(|chunk| chunk.so())
            .flat_map(|pairs| pairs.iter())
            .flat_map(|pair| pair.iter())
            .fold((u64::max_value(), 0), |acc, &x| {
                (acc.0.min(x), acc.1.max(x))
            });
        (min, max, (max - min + 1) as usize)
    }
}

/// Sort the pairs and remove duplicates
pub fn bucket_sort_pairs(
    pairs: &mut Vec<[u64; 2]>,
    hist: &mut Vec<usize>,
    hist2: &mut Vec<usize>,
    cumul: &mut Vec<usize>,
    min: u64,
    _max: u64,
    width: usize,
) -> usize {
    if pairs.is_empty() {
        return 0;
    }
    build_hist(pairs, min, hist);
    mem::replace(hist2, hist.to_vec());
    build_cumul(&hist, cumul);
    let len = pairs.len();
    let mut objects = vec![0; len];
    for i in 0..len {
        let pos = cumul[(pairs[i][0] - min) as usize];
        let remaining = hist[(pairs[i][0] - min) as usize];
        hist[(pairs[i][0] - min) as usize] -= 1;
        objects[(pos + remaining - 1) as usize] = pairs[i][1];
    }
    for i in 0..(width - 1) {
        insertion_sort_slice(&mut objects, cumul[i], cumul[i + 1]);
    }
    insertion_sort_slice(&mut objects, cumul[width - 1], len);
    let mut j = 0;
    let mut l = 0;
    let mut last = 0;
    for i in 0..width {
        let val = hist2[i];
        let s = min + i as u64;
        for k in 0..val {
            let o = objects[l];
            l += 1;
            if k == 0 || o != last {
                pairs[j] = [s, o];
                j += 1;
            }
            last = o;
        }
    }
    pairs.truncate(j);
    j
}

fn insertion_sort_slice(v: &mut [u64], from: usize, to: usize) {
    for i in from..to {
        let mut j = i;
        let tmp = v[i];
        while j > from && v[j - 1] > tmp {
            v[j] = v[j - 1];
            j -= 1;
        }
        v[j] = tmp;
    }
}

fn build_hist(pairs: &[[u64; 2]], min: u64, hist: &mut Vec<usize>) {
    for pair in pairs {
        hist[(pair[0] - min) as usize] += 1;
    }
}

fn build_cumul(hist: &[usize], cumul: &mut Vec<usize>) {
    for i in 1..hist.len() {
        cumul[i] = cumul[i - 1] + hist[i - 1];
    }
}
