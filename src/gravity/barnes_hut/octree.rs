/// SoA Barnes-Hut Octree
pub struct BarnesHutOctree {
    pub theta: f64, // opening angle threshold for Barnes-Hut approximation
    pub max_leaf_size: usize,

    // bodies
    pub body_masses: Vec<f64>,
    pub body_pos_x: Vec<f64>,
    pub body_pos_y: Vec<f64>,
    pub body_pos_z: Vec<f64>,

    // cache of the sorted morton codes for each body
    pub body_sorted_morton_codes: Vec<u64>,

    // permutation array storing the mapping from morton sorted body index to original body index
    pub body_permutations: Vec<usize>,

    // nodes
    pub node_masses: Vec<f64>,
    pub node_com_x: Vec<f64>,
    pub node_com_y: Vec<f64>,
    pub node_com_z: Vec<f64>,
    pub node_widths: Vec<f64>,

    // bitmask indicating which children exist for each node, where the 8 bits correspond to the 8 children (0 to 7)
    pub node_children_masks: Vec<u8>,

    // children_start_idx[i] points to the first of the contiguous existing children of node i in the node arrays.
    // If a node is a leaf, then u32::MAX is used as a sentinel value to indicate that it has no children.
    pub node_children_start_idx: Vec<u32>,

    // points to the start and length of the block in the sorted body arrays
    // due to Morton code sorting, all bodies in the same node will be
    // in [node_bodies_start[i], node_bodies_start[i]+node_bodies_count[i]) in the body arrays.
    pub node_bodies_start: Vec<u32>,
    pub node_bodies_count: Vec<u32>,

    pub node_count: usize,
}

impl BarnesHutOctree {
    pub fn new(n: usize, theta: f64, max_leaf_size: usize) -> Self {
        // upper bound on the possible number of nodes in the octree
        let max_nodes = 2 * n - 1;
        Self {
            theta: theta,
            max_leaf_size: max_leaf_size, // can be tuned for performance

            // body properties
            body_masses: vec![0.0; n],
            body_pos_x: vec![0.0; n],
            body_pos_y: vec![0.0; n],
            body_pos_z: vec![0.0; n],
            body_sorted_morton_codes: vec![0; n],
            body_permutations: vec![0; n],

            // node properties
            node_masses: vec![0.0; max_nodes],
            node_com_x: vec![0.0; max_nodes],
            node_com_y: vec![0.0; max_nodes],
            node_com_z: vec![0.0; max_nodes],
            node_widths: vec![0.0; max_nodes],
            node_children_masks: vec![0; max_nodes],
            node_children_start_idx: vec![u32::MAX; max_nodes], // u32::max as sentinel value for leaf nodes
            node_bodies_start: vec![0; max_nodes],
            node_bodies_count: vec![0; max_nodes],

            node_count: 0,
        }
    }

    pub fn build(&mut self, masses: &[f64], rx: &[f64], ry: &[f64], rz: &[f64]) {
        self.reset();

        self.body_masses.copy_from_slice(masses);
        self.body_pos_x.copy_from_slice(rx);
        self.body_pos_y.copy_from_slice(ry);
        self.body_pos_z.copy_from_slice(rz);
        self.body_permutations = morton_sort_bodies(
            &mut self.body_masses,
            &mut self.body_pos_x,
            &mut self.body_pos_y,
            &mut self.body_pos_z,
        );

        // Start build with root at bit level 20 since we are using u64 morton codes
        // with 21 bits per coordinate, so the highest bit level is 20 (0-indexed)
        self.build_recursive(0, masses.len(), 20);
    }

    pub fn compute_acceleration_for_body<F: Fn(f64, f64, f64, f64) -> (f64, f64, f64)>(
        &self,
        i: usize,
        // function for computing acceleration from a single source (mass, dx, dy, dz) -> (ax, ay, az)
        acceleration_function: F,
    ) -> (f64, f64, f64) {
        // to be implemented
        (0.0, 0.0, 0.0)
    }

    fn build_recursive(
        &mut self,
        bodies_range_start: usize,
        bodies_range_end: usize,
        bit_level: i32,
    ) -> u32 {
        let node_idx = self.create_new_node();
        let num_bodies = bodies_range_end - bodies_range_start;

        // base case: leaf node
        if num_bodies <= self.max_leaf_size || bit_level < 0 {
            self.node_bodies_start[node_idx] = bodies_range_start as u32;
            self.node_bodies_count[node_idx] = num_bodies as u32;
            return node_idx as u32;
        }

        // recursive case: internal node (8 children)
        let mut current_search_start = bodies_range_start;
        for child_id in 0..8 {
            let child_range_end =
                self.find_split_point(current_search_start, bodies_range_end, bit_level, child_id);

            // only create a child node if there are bodies in this child's range
            if child_range_end > current_search_start {
                let child_node_idx =
                    self.build_recursive(current_search_start, child_range_end, bit_level - 1);
                self.node_children_start_idx[node_idx] = child_node_idx + child_id as u32;
            }

            current_search_start = child_range_end;
        }

        node_idx as u32
    }

    fn create_new_node(&mut self) -> usize {
        let index = self.node_count;
        self.node_count += 1;
        index
    }

    /// Finds the index of the given child_id (0 to 7) of the node at the given bit_level,
    fn find_split_point(&self, start: usize, end: usize, bit_level: i32, child_id: u8) -> usize {
        let slice = &self.body_sorted_morton_codes[start..end];

        // partition_point uses binary search to get the index of the first child that does not belong to this child
        start
            + slice.partition_point(|&code| {
                // Right shift the code to make the 3 LSBs correspond to the given bit_level,
                // and then mask these LSBs to get the child_id bits (3 bits for 2^3 = 8 children).
                let extracted_child = (code >> (bit_level * 3)) & 0b111;
                extracted_child <= child_id as u64
            })
    }

    fn reset(&mut self) {
        self.node_count = 0;
    }
}

/// Find the bounding box of the bodies, and return minimum x, y, z coordinates and the width of the root node.
fn find_bounding_box(rx: &[f64], ry: &[f64], rz: &[f64]) -> (f64, f64, f64, f64) {
    let (mut min_x, mut max_x) = (rx[0], rx[0]);
    let (mut min_y, mut max_y) = (ry[0], ry[0]);
    let (mut min_z, mut max_z) = (rz[0], rz[0]);

    for ((&x, &y), &z) in rx.iter().zip(ry).zip(rz).skip(1) {
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }

        if y < min_y {
            min_y = y;
        }
        if y > max_y {
            max_y = y;
        }

        if z < min_z {
            min_z = z;
        }
        if z > max_z {
            max_z = z;
        }
    }

    let dx = max_x - min_x;
    let dy = max_y - min_y;
    let dz = max_z - min_z;

    // padding is added to ensure that all bodies are comfortably within the root node,
    // and to prevent edge cases where bodies lie exactly on the boundary of the root node,
    // which could cause issues with Morton code quantization and octree construction.
    let root_width = dx.max(dy).max(dz) * 1.0001;

    (min_x, min_y, min_z, root_width)
}

/// Quantize a f64 value to a u64 in the range [0, 2^21 - 1]
fn quantize_f64_to_u64(val: f64, min: f64, width: f64) -> u64 {
    let normalized = (val - min) / width;
    (normalized * (1 << 21) as f64) as u64
}

/// Sorts SoA bodies by morton code order and returns an array with the mapping from sorted to original indices
fn morton_sort_bodies(
    masses: &mut [f64],
    pos_x: &mut [f64],
    pos_y: &mut [f64],
    pos_z: &mut [f64],
) -> Vec<usize> {
    let n = masses.len();
    let (min_x, min_y, min_z, root_width) = find_bounding_box(pos_x, pos_y, pos_z);
    let mut morton_codes_and_idx: Vec<(u64, usize)> = (0..n)
        .map(|i| {
            let qx = quantize_f64_to_u64(pos_x[i], min_x, root_width);
            let qy = quantize_f64_to_u64(pos_y[i], min_y, root_width);
            let qz = quantize_f64_to_u64(pos_z[i], min_z, root_width);
            (morton_encode(qx, qy, qz), i)
        })
        .collect();

    morton_codes_and_idx.sort_unstable(); // TODO look into radix sort here

    let mut permutations = Vec::with_capacity(n);
    for (_, idx) in morton_codes_and_idx {
        permutations.push(idx);
    }

    let mut scratch = Vec::with_capacity(n); // reused to avoid repeated vector creations
    let mut reorder = |data: &mut [f64]| {
        scratch.clear();
        for &i in &permutations {
            scratch.push(data[i]);
        }
        data.copy_from_slice(&scratch);
    };

    // Apply the same permutations to all SoA arrays
    reorder(masses);
    reorder(pos_x);
    reorder(pos_y);
    reorder(pos_z);

    permutations
}

// Loop-based implementation of morton_encode for reference
//
// Interleave the bits of x, y, and z to create a single Morton code.
// fn morton_encode(x: u64, y: u64, z: u64) -> u64 {
//     let mut morton_code = 0;
//     for i in 0..21 {
//         // shift i-th bit ((z >> i) & 1) by (3 * i + offset)
//         morton_code |= ((z >> i) & 1) << (3 * i + 2);
//         morton_code |= ((y >> i) & 1) << (3 * i + 1);
//         morton_code |= ((x >> i) & 1) << (3 * i);
//     }
//     morton_code
// }

/// Interleave the bits of x, y, and z to create a single Morton code
fn morton_encode(x: u64, y: u64, z: u64) -> u64 {
    morton_spread_bits(x) | (morton_spread_bits(y) << 1) | (morton_spread_bits(z) << 2)
}

/// Uses the "magic bits" method to split the bits of x apart with 2 zeroes in between, so that they're in the correct positions
/// to be combined with the similarly spread and shifted bits of y and z to create the final Morton code.
/// See: https://www.forceflow.be/2013/10/07/morton-encodingdecoding-through-bit-interleaving-implementations/
fn morton_spread_bits(mut x: u64) -> u64 {
    x &= 0x1fffff; // ensure we only have 21 bits
    x = (x | (x << 32)) & 0x1f00000000ffff; // & mask from but 63: 00000000
    x = (x | (x << 16)) & 0x1f0000ff0000ff; //
    x = (x | (x << 8)) & 0x100f00f00f00f00f; // after this step, the bits of x are spread out with 8 zeros in between, and we keep only the relevant bits
    x = (x | (x << 4)) & 0x10c30c30c30c30c3; //
    x = (x | (x << 2)) & 0x1249249249249249; //
    x
}
