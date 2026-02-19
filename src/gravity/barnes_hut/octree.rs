// See /docs/barnes_hut_octree.md for design notes and implementation details.

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

    // Flattened list of all children for all nodes, where the children of each node are stored contiguously,
    // and the start index of each node's children in this array is given by node_children_start_idx.
    pub flat_node_children: Vec<usize>, // flattened list of all children for all nodes

    // flat_node_children_start_idx[i] points to the first of the contiguous existing children of node i in the node arrays.
    // If a node is a leaf, then usize::MAX is used as a sentinel value to indicate that it has no children.
    pub flat_node_children_start_idx: Vec<usize>,
    pub node_children_count: Vec<usize>, // number of children for each node, can be computed from node_children_masks but stored here for convenience

    // points to the start and length of the block in the sorted body arrays
    // due to Morton code sorting, all bodies in the same node will be
    // in [node_bodies_start[i], node_bodies_start[i]+node_bodies_count[i]) in the body arrays.
    pub node_bodies_start: Vec<usize>,
    pub node_bodies_count: Vec<usize>,

    pub node_count: usize,
}

impl BarnesHutOctree {
    // padding is applied to root width to ensure that all bodies are comfortably within the root node,
    // and to prevent edge cases where bodies lie exactly on the boundary of the root node,
    // which could cause issues with Morton code quantization and octree construction.
    const DEFAULT_PADDING: f64 = 1.001;

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
            flat_node_children: Vec::with_capacity(max_nodes), // reserving with capacity because this will be pushed to this during tree construction
            flat_node_children_start_idx: vec![usize::MAX; max_nodes], // usize::max as sentinel value for leaf nodes
            node_children_count: vec![0; max_nodes],
            node_bodies_start: vec![usize::MAX; max_nodes], // usize::max as sentinel value (should never be encountered for non-existent nodes)
            node_bodies_count: vec![0; max_nodes],

            node_count: 0,
        }
    }

    pub fn build(&mut self, masses: &[f64], rx: &[f64], ry: &[f64], rz: &[f64]) {
        self.reset();

        let (min_x, min_y, min_z, root_width) =
            find_bounding_box(rx, ry, rz, Self::DEFAULT_PADDING);

        self.body_masses.copy_from_slice(masses);
        self.body_pos_x.copy_from_slice(rx);
        self.body_pos_y.copy_from_slice(ry);
        self.body_pos_z.copy_from_slice(rz);

        (self.body_permutations, self.body_sorted_morton_codes) = morton_sort_bodies(
            &mut self.body_masses,
            &mut self.body_pos_x,
            &mut self.body_pos_y,
            &mut self.body_pos_z,
            min_x,
            min_y,
            min_z,
            root_width,
        );

        // Start build with root at bit level 20 since we are using u64 morton codes
        // with 21 bits per coordinate, so the highest bit level is 20 (0-indexed)
        self.build_recursive(0, masses.len(), 20, root_width);
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

    /// Recursively builds the octree by computing node indices and per-node body tracking information
    fn build_recursive(
        &mut self,
        bodies_range_start_idx: usize,
        bodies_range_end_idx: usize,
        bit_level: i32,
        width: f64,
    ) -> usize {
        let node_idx = self.create_new_node();
        let num_bodies = bodies_range_end_idx - bodies_range_start_idx;
        self.node_bodies_start[node_idx] = bodies_range_start_idx;
        self.node_bodies_count[node_idx] = num_bodies;
        self.node_widths[node_idx] = width;

        // base case: leaf node
        if num_bodies <= self.max_leaf_size || bit_level < 0 {
            let (mass, com_x, com_y, com_z) =
                self.compute_com_of_bodies_range(bodies_range_start_idx, bodies_range_end_idx);
            self.node_masses[node_idx] = mass;
            self.node_com_x[node_idx] = com_x;
            self.node_com_y[node_idx] = com_y;
            self.node_com_z[node_idx] = com_z;

            // Print node information during tests
            #[cfg(test)]
            {
                println!(
                    "Node {} (leaf): bit_level={}, num_bodies={}, mass={}, com=({}, {}, {})",
                    node_idx, bit_level, num_bodies, mass, com_x, com_y, com_z
                );
                println!(
                    "\tBodies range [{}, {})",
                    bodies_range_start_idx, bodies_range_end_idx
                );
            }

            return node_idx;
        }

        // recursive case: internal node (split into up to 8 children)
        let mut children_count = 0;
        let mut current_search_start_idx = bodies_range_start_idx;
        for child_id in 0..8 {
            let child_range_end_idx = self.find_split_point(
                current_search_start_idx,
                bodies_range_end_idx,
                bit_level,
                child_id,
            );

            // if there are no bodies in the potential child's range, child_range_end_idx == current_search_start_idx,
            // so only creating a child node if there are bodies in this child's range
            if child_range_end_idx > current_search_start_idx {
                let child_node_idx = self.build_recursive(
                    current_search_start_idx,
                    child_range_end_idx,
                    bit_level - 1,
                    width * 0.5,
                );
                self.flat_node_children.push(child_node_idx);
                children_count += 1;
            }
            current_search_start_idx = child_range_end_idx;
        }

        // The previous loop pushes the children of this node to the end of flat_node_children,
        //  the start index of this node's children is the current length of flat_node_children minus the number of children we just added.
        let children_start = self.flat_node_children.len() - children_count;

        self.flat_node_children_start_idx[node_idx] = children_start;
        self.node_children_count[node_idx] = children_count;

        // Aggregate mass/COM from children
        let (mass, com_x, com_y, com_z) = if children_count > 0 {
            let start = children_start;
            let end = children_start + children_count;
            self.compute_com_of_node_indices(&self.flat_node_children[start..end])
        } else {
            (0.0, 0.0, 0.0, 0.0)
        };

        self.node_masses[node_idx] = mass;
        self.node_com_x[node_idx] = com_x;
        self.node_com_y[node_idx] = com_y;
        self.node_com_z[node_idx] = com_z;

        // Print node information during tests
        #[cfg(test)]
        {
            println!(
                "Node {} (internal): bit_level={}, num_bodies={}, children_count={}, mass={}, com=({}, {}, {})",
                node_idx, bit_level, num_bodies, children_count, mass, com_x, com_y, com_z
            );
            println!(
                "\tFlat children nodes range [{},{}), Children indices {:?}, Bodies range [{}, {})",
                children_start,
                children_start + children_count,
                &self.flat_node_children[children_start..children_start + children_count],
                bodies_range_start_idx,
                bodies_range_end_idx
            );
        }

        node_idx
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

    fn compute_com_of_bodies_range(
        &self,
        start_idx: usize,
        end_idx: usize,
    ) -> (f64, f64, f64, f64) {
        compute_com(
            &self.body_masses[start_idx..end_idx],
            &self.body_pos_x[start_idx..end_idx],
            &self.body_pos_y[start_idx..end_idx],
            &self.body_pos_z[start_idx..end_idx],
        )
    }

    fn compute_com_of_node_indices(&self, indices: &[usize]) -> (f64, f64, f64, f64) {
        // TODO convert node index tracking to SoA to avoid this repeated indexing and vector creation during tree construction
        let node_masses = indices
            .iter()
            .map(|&i| self.node_masses[i])
            .collect::<Vec<_>>();
        let node_com_x = indices
            .iter()
            .map(|&i| self.node_com_x[i])
            .collect::<Vec<_>>();
        let node_com_y = indices
            .iter()
            .map(|&i| self.node_com_y[i])
            .collect::<Vec<_>>();
        let node_com_z = indices
            .iter()
            .map(|&i| self.node_com_z[i])
            .collect::<Vec<_>>();
        compute_com(&node_masses, &node_com_x, &node_com_y, &node_com_z)
    }
}

/// Find the bounding box of the bodies, and return minimum x, y, z coordinates and the width of the root node.
fn find_bounding_box(rx: &[f64], ry: &[f64], rz: &[f64], padding: f64) -> (f64, f64, f64, f64) {
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

    let root_width = dx.max(dy).max(dz) * padding;

    (min_x, min_y, min_z, root_width)
}

/// Quantize a f64 value to a u64 in the range [0, 2^21 - 1]
fn quantize_f64_to_u64(val: f64, min: f64, width: f64) -> u64 {
    let normalized = (val - min) / width;
    (normalized * (1 << 21) as f64) as u64
}

/// Sorts SoA bodies by morton code order and returns
///  - a permutation array with the mapping from sorted to original indices
///  - the sorted morton codes for each body
fn morton_sort_bodies(
    masses: &mut [f64],
    pos_x: &mut [f64],
    pos_y: &mut [f64],
    pos_z: &mut [f64],
    min_x: f64,
    min_y: f64,
    min_z: f64,
    root_width: f64,
) -> (Vec<usize>, Vec<u64>) {
    let n = masses.len();

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
    let mut sorted_morton_codes = Vec::with_capacity(n);
    for (code, idx) in morton_codes_and_idx.iter() {
        sorted_morton_codes.push(*code);
        permutations.push(*idx);
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

    (permutations, sorted_morton_codes)
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

/// Compute total mass and CoM of of a set of masses and positions
fn compute_com(masses: &[f64], xs: &[f64], ys: &[f64], zs: &[f64]) -> (f64, f64, f64, f64) {
    let mut total_mass = 0.0;
    let mut com_x = 0.0;
    let mut com_y = 0.0;
    let mut com_z = 0.0;

    for (((&mass, &x), &y), &z) in masses.iter().zip(xs).zip(ys).zip(zs) {
        total_mass += mass;
        com_x += mass * x;
        com_y += mass * y;
        com_z += mass * z;
    }

    if total_mass > 0.0 {
        com_x /= total_mass;
        com_y /= total_mass;
        com_z /= total_mass;
    }

    (total_mass, com_x, com_y, com_z)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_body_tree() {
        let masses = vec![2.0];
        let xs = vec![1.0];
        let ys = vec![2.0];
        let zs = vec![3.0];
        let mut tree = BarnesHutOctree::new(1, 0.5, 1);
        tree.build(&masses, &xs, &ys, &zs);

        // Only one node (the root/leaf)
        assert_eq!(tree.node_count, 1);
        assert_eq!(tree.node_masses[0], 2.0);
        assert_eq!(tree.node_com_x[0], 1.0);
        assert_eq!(tree.node_com_y[0], 2.0);
        assert_eq!(tree.node_com_z[0], 3.0);
        assert_eq!(tree.node_bodies_count[0], 1);
        assert_eq!(tree.node_children_count[0], 0); // no children
        assert_eq!(tree.flat_node_children_start_idx[0], usize::MAX); // sentinel for leaf
    }

    #[test]
    fn test_two_body_tree() {
        let masses = vec![1.0, 3.0];
        let xs = vec![0.0, 1.0];
        let ys = vec![1.0, 0.0];
        let zs = vec![0.0, 0.0];
        let mut tree = BarnesHutOctree::new(2, 0.5, 1);
        tree.build(&masses, &xs, &ys, &zs);

        // Find all leaves (nodes with node_children_start_idx == usize::MAX)
        let leaves: Vec<_> = (0..tree.node_count)
            .filter(|&i| tree.flat_node_children_start_idx[i] == usize::MAX)
            .collect();
        assert_eq!(leaves.len(), 2, "Should have 2 leaves for 2 bodies");

        // Each leaf should have one body, and COM should match
        let mut found = [false; 2];
        for &leaf_idx in &leaves {
            assert_eq!(tree.node_bodies_count[leaf_idx], 1);
            let x = tree.node_com_x[leaf_idx];
            let y = tree.node_com_y[leaf_idx];
            let z = tree.node_com_z[leaf_idx];
            if (x - 0.0).abs() < 1e-12 && (y - 1.0).abs() < 1e-12 && (z - 0.0).abs() < 1e-12 {
                found[0] = true;
            } else if (x - 1.0).abs() < 1e-12 && (y - 0.0).abs() < 1e-12 && (z - 0.0).abs() < 1e-12
            {
                found[1] = true;
            }
        }
        assert!(found.iter().all(|&b| b), "Both leaves found and correct");

        // Internal nodes (just root) should have two bodies total
        assert_eq!(
            tree.node_bodies_count[0], 2,
            "Root should have 2 bodies total"
        );

        // Root node: mass and COM should be aggregate
        let total_mass = 1.0 + 3.0;
        let expected_com_x = (1.0 * 0.0 + 3.0 * 1.0) / total_mass;
        let expected_com_y = (1.0 * 1.0 + 3.0 * 0.0) / total_mass;
        assert_eq!(tree.node_masses[0], total_mass);
        assert!((tree.node_com_x[0] - expected_com_x).abs() < 1e-12);
        assert!((tree.node_com_y[0] - expected_com_y).abs() < 1e-12);

        // Root node's body count should equal total bodies (test-only)
        #[cfg(test)]
        assert_eq!(tree.node_bodies_count[0], 2);
    }

    #[test]
    fn test_three_body_tree() {
        let masses = vec![1.0, 2.0, 3.0];
        let xs = vec![0.0, 1.0, 2.0];
        let ys = vec![0.0, 0.0, 0.0];
        let zs = vec![0.0, 0.0, 0.0];
        let mut tree = BarnesHutOctree::new(3, 0.5, 1);
        tree.build(&masses, &xs, &ys, &zs);

        // Find all leaves
        let leaves: Vec<_> = (0..tree.node_count)
            .filter(|&i| tree.flat_node_children_start_idx[i] == usize::MAX)
            .collect();
        assert_eq!(leaves.len(), 3, "Should have 3 leaves for 3 bodies");

        // Each leaf should have one body, and COM should match
        let mut found = [false; 3];
        for &leaf_idx in &leaves {
            assert_eq!(tree.node_bodies_count[leaf_idx], 1);
            let x = tree.node_com_x[leaf_idx];
            if (x - 0.0).abs() < 1e-12 {
                found[0] = true;
            } else if (x - 1.0).abs() < 1e-12 {
                found[1] = true;
            } else if (x - 2.0).abs() < 1e-12 {
                found[2] = true;
            }
        }
        assert!(found.iter().all(|&b| b), "All three leaves found");

        // Internal nodes should have three bodies total
        assert_eq!(
            tree.node_bodies_count[0], 3,
            "Root should have 3 bodies total"
        );

        // Root node: mass and COM should be aggregate
        let total_mass = 1.0 + 2.0 + 3.0;
        let expected_com_x = (1.0 * 0.0 + 2.0 * 1.0 + 3.0 * 2.0) / total_mass;
        assert_eq!(tree.node_masses[0], total_mass);
        assert!((tree.node_com_x[0] - expected_com_x).abs() < 1e-12);

        #[cfg(test)]
        assert_eq!(tree.node_bodies_count[0], 3);
    }
}
