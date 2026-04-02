use std::collections::{BTreeMap, HashMap, HashSet};

pub const PAGE_SIZE: usize = 4096;
const CHUNK_SIZE: usize = 64;
const CHUNKS_PER_PAGE: usize = PAGE_SIZE / CHUNK_SIZE;
const MINHASHES: usize = 4;
const DELTA_HEADER_BYTES: usize = 6;

#[derive(Clone)]
pub struct PageWorkload {
    pub name: &'static str,
    pub pages: Vec<[u8; PAGE_SIZE]>,
}

#[derive(Default, Clone, Debug)]
pub struct MemoryReport {
    pub page_count: usize,
    pub unique_pages: usize,
    pub family_count: usize,
    pub exact_duplicate_pages: usize,
    pub near_duplicate_pages: usize,
    pub raw_bytes: usize,
    pub estimated_bytes: usize,
    pub exact_saved_bytes: usize,
    pub delta_saved_bytes: usize,
    pub best_delta_savings_percent: f64,
    pub avg_chunk_similarity_percent: f64,
}

impl MemoryReport {
    pub fn savings_percent(&self) -> f64 {
        100.0 * (self.raw_bytes.saturating_sub(self.estimated_bytes)) as f64 / self.raw_bytes as f64
    }

    pub fn compressibility_ratio(&self) -> f64 {
        self.raw_bytes as f64 / self.estimated_bytes.max(1) as f64
    }
}

#[derive(Clone)]
struct PageInfo {
    index: usize,
    hash: u64,
    signature: [u64; MINHASHES],
}

pub fn analyze_workload(workload: &PageWorkload) -> MemoryReport {
    let page_count = workload.pages.len();
    let raw_bytes = page_count * PAGE_SIZE;

    let infos: Vec<PageInfo> = workload
        .pages
        .iter()
        .enumerate()
        .map(|(index, page)| PageInfo {
            index,
            hash: fnv64(page),
            signature: page_signature(page),
        })
        .collect();

    let mut exact_groups: HashMap<u64, Vec<usize>> = HashMap::new();
    for info in &infos {
        exact_groups.entry(info.hash).or_default().push(info.index);
    }

    let mut representative_indices = Vec::new();
    let mut exact_duplicate_pages = 0;
    let mut exact_saved_bytes = 0;
    for group in exact_groups.values() {
        representative_indices.push(group[0]);
        if group.len() > 1 {
            exact_duplicate_pages += group.len() - 1;
            exact_saved_bytes += (group.len() - 1) * PAGE_SIZE;
        }
    }

    let mut buckets: BTreeMap<[u64; MINHASHES], Vec<usize>> = BTreeMap::new();
    for info in &infos {
        buckets.entry(info.signature).or_default().push(info.index);
    }

    let mut family_count = 0;
    let mut near_duplicate_pages = 0;
    let mut delta_saved_bytes = 0;
    let mut best_delta_savings_percent = 0.0;
    let mut total_similarity = 0.0;
    let mut similarity_samples = 0_usize;
    let mut claimed = HashSet::new();

    for bucket in buckets.values() {
        if bucket.len() < 2 {
            continue;
        }
        family_count += 1;
        let base = bucket[0];
        for &candidate in &bucket[1..] {
            if exact_groups[&infos[candidate].hash].len() > 1 || claimed.contains(&candidate) {
                continue;
            }
            let similarity = chunk_similarity(&workload.pages[base], &workload.pages[candidate]);
            total_similarity += similarity;
            similarity_samples += 1;
            if similarity < 0.70 {
                continue;
            }
            let delta_cost = delta_encoded_size(&workload.pages[base], &workload.pages[candidate]);
            if delta_cost >= PAGE_SIZE {
                continue;
            }
            let saved = PAGE_SIZE - delta_cost;
            near_duplicate_pages += 1;
            delta_saved_bytes += saved;
            claimed.insert(candidate);
            let savings_percent = 100.0 * saved as f64 / PAGE_SIZE as f64;
            if savings_percent > best_delta_savings_percent {
                best_delta_savings_percent = savings_percent;
            }
        }
    }

    let unique_pages = page_count - exact_duplicate_pages - near_duplicate_pages;
    let estimated_bytes = raw_bytes - exact_saved_bytes - delta_saved_bytes;
    MemoryReport {
        page_count,
        unique_pages,
        family_count,
        exact_duplicate_pages,
        near_duplicate_pages,
        raw_bytes,
        estimated_bytes,
        exact_saved_bytes,
        delta_saved_bytes,
        best_delta_savings_percent,
        avg_chunk_similarity_percent: if similarity_samples == 0 {
            0.0
        } else {
            100.0 * total_similarity / similarity_samples as f64
        },
    }
}

pub fn exact_duplicate_workload() -> PageWorkload {
    let mut pages = Vec::new();
    let base = page_from_seed(7, 0x00);
    for _ in 0..600 {
        pages.push(base);
    }
    for i in 0..200 {
        pages.push(page_from_seed(i as u64, 0x5a));
    }
    PageWorkload {
        name: "exact_dups",
        pages,
    }
}

pub fn near_duplicate_workload() -> PageWorkload {
    let mut pages = Vec::new();
    for family in 0..64_u64 {
        let base = page_from_seed(family, 0x33);
        pages.push(base);
        for variant in 0..7_u8 {
            let mut page = base;
            mutate_window(
                &mut page,
                variant as usize * 128,
                96,
                family as u8 ^ variant,
            );
            mutate_window(&mut page, 1024 + variant as usize * 32, 16, variant);
            pages.push(page);
        }
    }
    PageWorkload {
        name: "near_dups",
        pages,
    }
}

pub fn mixed_workload() -> PageWorkload {
    let mut pages = Vec::new();
    let exact = exact_duplicate_workload();
    pages.extend(exact.pages.into_iter().take(300));
    let near = near_duplicate_workload();
    pages.extend(near.pages.into_iter().take(256));
    for i in 0..300 {
        pages.push(page_from_seed(10_000 + i as u64, 0xa7));
    }
    PageWorkload {
        name: "mixed",
        pages,
    }
}

fn delta_encoded_size(base: &[u8; PAGE_SIZE], other: &[u8; PAGE_SIZE]) -> usize {
    let mut encoded = 0_usize;
    let mut idx = 0;
    while idx < PAGE_SIZE {
        if base[idx] == other[idx] {
            idx += 1;
            continue;
        }
        let start = idx;
        while idx < PAGE_SIZE && base[idx] != other[idx] {
            idx += 1;
        }
        encoded += DELTA_HEADER_BYTES + (idx - start);
    }
    encoded
}

fn chunk_similarity(a: &[u8; PAGE_SIZE], b: &[u8; PAGE_SIZE]) -> f64 {
    let mut equal = 0_usize;
    for chunk in 0..CHUNKS_PER_PAGE {
        let start = chunk * CHUNK_SIZE;
        let end = start + CHUNK_SIZE;
        if a[start..end] == b[start..end] {
            equal += 1;
        }
    }
    equal as f64 / CHUNKS_PER_PAGE as f64
}

fn page_signature(page: &[u8; PAGE_SIZE]) -> [u64; MINHASHES] {
    let mut mins = [u64::MAX; MINHASHES];
    for chunk in page.chunks(CHUNK_SIZE) {
        let hash = fnv64(chunk);
        for slot in &mut mins {
            if hash < *slot {
                let current = *slot;
                *slot = hash;
                bubble_down(&mut mins);
                if current == u64::MAX {
                    break;
                }
                break;
            }
        }
    }
    mins.sort_unstable();
    mins
}

fn bubble_down(values: &mut [u64; MINHASHES]) {
    for idx in 1..values.len() {
        if values[idx] < values[idx - 1] {
            values.swap(idx, idx - 1);
        }
    }
}

fn fnv64(data: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn page_from_seed(seed: u64, salt: u8) -> [u8; PAGE_SIZE] {
    let mut page = [0_u8; PAGE_SIZE];
    let mut state = seed ^ ((salt as u64) << 32);
    for byte in &mut page {
        state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *byte = ((state >> 24) as u8) ^ salt;
    }
    page
}

fn mutate_window(page: &mut [u8; PAGE_SIZE], start: usize, len: usize, marker: u8) {
    for idx in start..start + len {
        page[idx] ^= marker.wrapping_add((idx - start) as u8);
    }
}

#[cfg(test)]
mod tests {
    use super::{analyze_workload, mixed_workload, near_duplicate_workload};

    #[test]
    fn mixed_workload_saves_material_memory() {
        let report = analyze_workload(&mixed_workload());
        assert!(report.savings_percent() > 40.0);
    }

    #[test]
    fn near_duplicate_workload_detects_families() {
        let report = analyze_workload(&near_duplicate_workload());
        assert!(report.family_count > 10);
        assert!(report.near_duplicate_pages > 100);
    }
}
