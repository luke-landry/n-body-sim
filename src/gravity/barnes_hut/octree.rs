/*
   Implementation of a Barnes-Hut octree for 3D N-body simulations
   using a Structure of Arrays (SoA) data layout with morton code ordering.

   See /docs/barnes_hut_octree.md for design notes
*/

/// SoA Barnes-Hut Octree
pub struct BarnesHutOctree {
    theta: f64, // threshold for Barnes-Hut approximation
    max_leaf_size: usize,
    node_count: usize,

    // bodies
    body_masses: Vec<f64>,
    body_pos_x: Vec<f64>,
    body_pos_y: Vec<f64>,
    body_pos_z: Vec<f64>,

    /// cache of the sorted morton codes for each body
    body_sorted_morton_codes: Vec<u64>,

    /// array storing the mapping from sorted body index to original body index
    /// original_to_morton_idx[original_idx] = morton_idx
    original_to_morton_idx: Vec<usize>,

    /// array storing the mapping from morton sorted body index to original body index
    /// morton_to_original_idx[morton_idx] = original_idx
    morton_to_original_idx: Vec<usize>,

    // nodes
    node_masses: Vec<f64>,
    node_com_x: Vec<f64>,
    node_com_y: Vec<f64>,
    node_com_z: Vec<f64>,
    node_widths: Vec<f64>,

    // per-node indices of children during tree construction, will be flattened into flat_node_children after construction
    node_children: Vec<Vec<usize>>, // TODO replace with vec[max_nodes  * 8] for fully contiguous memory layout
    node_children_count: Vec<usize>,

    /// Points to the start and length of the block in the sorted body arrays.
    /// Due to Morton code sorting, all bodies in the same node will be
    /// in [node_bodies_start[i], node_bodies_start[i]+node_bodies_count[i]) in the body arrays.
    node_bodies_start: Vec<usize>,
    node_bodies_count: Vec<usize>,

    /// Flattened list of sorted children node indices, where the children of each node are stored contiguously,
    /// and the start index of each node's children in this array is given by node_children_start_idx.
    flat_node_children: Vec<usize>,

    /// flat_node_children_start_idx[i] points to the first of the contiguous existing children of node i in the node arrays.
    /// If a node is a leaf, then usize::MAX is used as a sentinel value to indicate that it has no children.
    flat_node_children_start_idx: Vec<usize>,
}

impl BarnesHutOctree {
    // maximum depth of the octree, determined by the number of bits used
    // for u64 Morton codes (64 / 3 ~= 21 bits per 3D coordinate for 64-bit Morton codes)
    const MAX_DEPTH: usize = 21;

    // padding is applied to root width to ensure that all bodies are comfortably within the root node,
    // and to prevent edge cases where bodies lie exactly on the boundary of the root node,
    // which could cause issues with Morton code quantization and octree construction.
    const DEFAULT_PADDING: f64 = 1.001;

    pub fn new(n: usize, theta: f64, max_leaf_size: usize) -> Self {
        // upper bound on the possible number of nodes in the octree
        // assuming a maximum depth of 21 since we are using 21 bits per coordinate for the u64 Morton codes
        // T < (N/C) * (1 + D) where N is the number of bodies, C is the max leaf size, and D is the maximum depth of the tree
        let max_nodes = (n.div_ceil(max_leaf_size)) * (1 + Self::MAX_DEPTH);

        Self {
            theta: theta,
            max_leaf_size: max_leaf_size, // can be tuned for performance
            node_count: 0,

            // body properties
            body_masses: vec![0.0; n],
            body_pos_x: vec![0.0; n],
            body_pos_y: vec![0.0; n],
            body_pos_z: vec![0.0; n],
            body_sorted_morton_codes: vec![0; n],
            original_to_morton_idx: vec![0; n],
            morton_to_original_idx: vec![0; n],

            // node properties
            node_masses: vec![0.0; max_nodes],
            node_com_x: vec![0.0; max_nodes],
            node_com_y: vec![0.0; max_nodes],
            node_com_z: vec![0.0; max_nodes],
            node_widths: vec![0.0; max_nodes],
            node_children: vec![Vec::with_capacity(8); max_nodes], // each node can have up to 8 children in an octree
            node_children_count: vec![0; max_nodes],
            node_bodies_start: vec![usize::MAX; max_nodes], // usize::max as sentinel value (should never be encountered for non-existent nodes)
            node_bodies_count: vec![0; max_nodes],

            flat_node_children: Vec::with_capacity(max_nodes), // reserving with capacity because this will be pushed to this during tree construction
            flat_node_children_start_idx: vec![usize::MAX; max_nodes], // usize::max as sentinel value for leaf nodes
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

        (
            self.original_to_morton_idx,
            self.morton_to_original_idx,
            self.body_sorted_morton_codes,
        ) = morton_sort_bodies(
            &mut self.body_masses,
            &mut self.body_pos_x,
            &mut self.body_pos_y,
            &mut self.body_pos_z,
            min_x,
            min_y,
            min_z,
            root_width,
        );

        self.build_recursive(0, masses.len(), Self::MAX_DEPTH, root_width);

        // Flatten the node_children into flat_node_children and set flat_node_children_start_idx
        for node_idx in 0..self.node_count {
            let children = &self.node_children[node_idx];
            if !children.is_empty() {
                self.flat_node_children_start_idx[node_idx] = self.flat_node_children.len();
                self.flat_node_children.extend_from_slice(children);
            }
        }
    }

    pub fn compute_acceleration_for_body<F: Fn(f64, f64, f64, f64) -> (f64, f64, f64)>(
        &self,
        // target body index in the original input order (not morton sorted order)
        target_body_idx: usize,
        // function for computing acceleration from a single source (mass, dx, dy, dz) -> (ax, ay, az)
        acceleration_function: F,
    ) -> (f64, f64, f64) {
        let mut ax = 0.0;
        let mut ay = 0.0;
        let mut az = 0.0;

        let target_body_morton_idx = self.original_to_morton_idx[target_body_idx];
        let target_body_pos_x = self.body_pos_x[target_body_morton_idx];
        let target_body_pos_y = self.body_pos_y[target_body_morton_idx];
        let target_body_pos_z = self.body_pos_z[target_body_morton_idx];

        #[cfg(test)]
        {
            println!(
                "Computing acceleration for body {}: morton_idx={}, pos=({}, {}, {})",
                target_body_idx,
                target_body_morton_idx,
                target_body_pos_x,
                target_body_pos_y,
                target_body_pos_z
            );
        }

        // using iteration for tree traversal instead of recursion for better performance
        let mut stack = vec![0]; // start with root node index on the stack

        while let Some(node_idx) = stack.pop() {
            let node_dx = self.node_com_x[node_idx] - target_body_pos_x;
            let node_dy = self.node_com_y[node_idx] - target_body_pos_y;
            let node_dz = self.node_com_z[node_idx] - target_body_pos_z;
            let node_distance_squared = node_dx * node_dx + node_dy * node_dy + node_dz * node_dz;
            if node_distance_squared == 0.0 {
                continue; // skip interaction with mass at the same position to avoid singularity
            }

            #[cfg(test)]
            {
                let bh_ratio = self.node_widths[node_idx] / node_distance_squared.sqrt();
                println!(
                    "Visiting node {}: node_com=({}, {}, {}), node_mass={}, node_width={}, distance={}, BH ratio={}, theta={}, leaf={}, num_bodies_in_node={}",
                    node_idx,
                    self.node_com_x[node_idx],
                    self.node_com_y[node_idx],
                    self.node_com_z[node_idx],
                    self.node_masses[node_idx],
                    self.node_widths[node_idx],
                    node_distance_squared.sqrt(),
                    bh_ratio,
                    self.theta,
                    self.node_children_count[node_idx] == 0,
                    self.node_bodies_count[node_idx],
                );
            }

            // If node is a leaf
            if self.node_children_count[node_idx] == 0 {
                // compute direct interaction with each body in this leaf node
                let bodies_start = self.node_bodies_start[node_idx];
                let bodies_end = bodies_start + self.node_bodies_count[node_idx];
                for i in bodies_start..bodies_end {
                    if i == target_body_morton_idx {
                        continue; // skip self-interaction
                    }
                    let source_body_mass = self.body_masses[i];
                    let source_body_dx = self.body_pos_x[i] - target_body_pos_x;
                    let source_body_dy = self.body_pos_y[i] - target_body_pos_y;
                    let source_body_dz = self.body_pos_z[i] - target_body_pos_z;
                    let (dax, day, daz) = acceleration_function(
                        source_body_mass,
                        source_body_dx,
                        source_body_dy,
                        source_body_dz,
                    );
                    ax += dax;
                    ay += day;
                    az += daz;
                }
            }
            /*
               Barnes-Hut criterion: if the node is sufficiently far away (width / distance < theta),
               we can approximate the entire node as a single mass at its center of mass.

               Squared calculations here as distance has already been squared:
               width_squared / distance_squared < theta^2 is equivalent to width / distance < theta
               given all these values are positive
            */
            else if ((self.node_widths[node_idx] * self.node_widths[node_idx])
                / node_distance_squared)
                < (self.theta * self.theta)
            {
                // case node is sufficiently far away to approximate as a single mass
                let node_mass = self.node_masses[node_idx];
                let (dax, day, daz) = acceleration_function(node_mass, node_dx, node_dy, node_dz);
                ax += dax;
                ay += day;
                az += daz;

                #[cfg(test)]
                {
                    println!(
                        "\tApproximated node {} as single mass:\n\t\twidth={}, distance={}, theta={}, node_mass={}, node_com=({}, {}, {})",
                        node_idx,
                        self.node_widths[node_idx],
                        node_distance_squared.sqrt(),
                        self.theta,
                        self.node_masses[node_idx],
                        self.node_com_x[node_idx],
                        self.node_com_y[node_idx],
                        self.node_com_z[node_idx],
                    );
                }
            }
            // If node is an internal node, traverse its children
            else {
                let children_start_idx = self.flat_node_children_start_idx[node_idx];
                let children_count = self.node_children_count[node_idx];
                let children_end_idx = children_start_idx + children_count;
                for i in children_start_idx..children_end_idx {
                    stack.push(self.flat_node_children[i]);
                }
            }
        }
        (ax, ay, az)
    }

    /// Recursively builds the octree by computing node indices and per-node body tracking information
    fn build_recursive(
        &mut self,
        bodies_range_start_idx: usize,
        bodies_range_end_idx: usize,
        bit_level: usize,
        width: f64,
    ) -> usize {
        let node_idx = self.create_new_node();
        let num_bodies = bodies_range_end_idx - bodies_range_start_idx;
        self.node_bodies_start[node_idx] = bodies_range_start_idx;
        self.node_bodies_count[node_idx] = num_bodies;
        self.node_widths[node_idx] = width;

        // base case: leaf node
        if num_bodies <= self.max_leaf_size || bit_level <= 0 {
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

        // recursive case: internal node (split into child nodes)
        let mut child_current_body_idx = bodies_range_start_idx;
        let shift = ((bit_level - 1) * 3) as u32;
        while child_current_body_idx < bodies_range_end_idx {
            // the morton code of the first body in this child's range determines which child node we are in (3 bits for 8 children)
            let child_node_first_body_morton_code =
                self.body_sorted_morton_codes[child_current_body_idx];
            let child_node_bit_level_id = (child_node_first_body_morton_code >> shift) & 0b111;

            // finding the end of this child's range by iterating until we find a body that does not belong to this child
            let mut child_bodies_range_end_idx = child_current_body_idx;
            while child_bodies_range_end_idx < bodies_range_end_idx {
                let body_morton_code = self.body_sorted_morton_codes[child_bodies_range_end_idx];

                // Right shift the code to make the 3 LSBs correspond to the given bit_level,
                // and then mask these LSBs to get the child_id bits (3 bits for 2^3 = 8 children).
                let body_bit_level_id = (body_morton_code >> shift) & 0b111;

                if body_bit_level_id != child_node_bit_level_id {
                    // this body does not belong to this child node so we have
                    // found the end of this child's range
                    break;
                }
                child_bodies_range_end_idx += 1;
            }

            // only create a child node if there are bodies in this child's range
            if child_bodies_range_end_idx > child_current_body_idx {
                let child_node_idx = self.build_recursive(
                    child_current_body_idx,
                    child_bodies_range_end_idx,
                    bit_level - 1,
                    width * 0.5,
                );
                self.node_children[node_idx].push(child_node_idx);
            }

            child_current_body_idx = child_bodies_range_end_idx;
        }

        let node_children = &self.node_children[node_idx];
        let node_children_count = node_children.len();

        self.node_children_count[node_idx] = node_children_count;

        // Aggregate mass/COM from children
        let (mass, com_x, com_y, com_z) = if node_children_count > 0 {
            self.compute_com_of_node_indices(&node_children)
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
                "Node {} (internal): bit_level={}, num_bodies={}, node_children_count={}, mass={}, com=({}, {}, {})",
                node_idx, bit_level, num_bodies, node_children_count, mass, com_x, com_y, com_z
            );
            println!(
                "\tChildren indices {:?}, Bodies range [{}, {})",
                &node_children, bodies_range_start_idx, bodies_range_end_idx
            );
        }

        node_idx
    }

    fn create_new_node(&mut self) -> usize {
        let index = self.node_count;
        self.node_count += 1;
        index
    }

    fn reset(&mut self) {
        self.body_masses.fill(0.0);
        self.body_pos_x.fill(0.0);
        self.body_pos_y.fill(0.0);
        self.body_pos_z.fill(0.0);
        self.body_sorted_morton_codes.fill(0);
        self.morton_to_original_idx.fill(0);
        self.node_masses.fill(0.0);
        self.node_com_x.fill(0.0);
        self.node_com_y.fill(0.0);
        self.node_com_z.fill(0.0);
        self.node_widths.fill(0.0);
        self.node_children
            .iter_mut()
            .for_each(|children| children.clear());
        self.node_children_count.fill(0);
        self.node_bodies_start.fill(usize::MAX);
        self.node_bodies_count.fill(0);
        self.flat_node_children.clear();
        self.flat_node_children_start_idx.fill(usize::MAX);
        self.node_count = 0;
    }

    fn compute_com_of_bodies_range(
        &self,
        start_idx: usize,
        end_idx: usize,
    ) -> (f64, f64, f64, f64) {
        let mut total_mass = 0.0;
        let mut com_x = 0.0;
        let mut com_y = 0.0;
        let mut com_z = 0.0;

        for (((&mass, &x), &y), &z) in self.body_masses[start_idx..end_idx]
            .iter()
            .zip(&self.body_pos_x[start_idx..end_idx])
            .zip(&self.body_pos_y[start_idx..end_idx])
            .zip(&self.body_pos_z[start_idx..end_idx])
        {
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

    fn compute_com_of_node_indices(&self, indices: &[usize]) -> (f64, f64, f64, f64) {
        let mut total_mass = 0.0;
        let mut com_x = 0.0;
        let mut com_y = 0.0;
        let mut com_z = 0.0;

        for &i in indices {
            let mass = self.node_masses[i];
            total_mass += mass;
            com_x += mass * self.node_com_x[i];
            com_y += mass * self.node_com_y[i];
            com_z += mass * self.node_com_z[i];
        }

        if total_mass > 0.0 {
            com_x /= total_mass;
            com_y /= total_mass;
            com_z /= total_mass;
        }

        (total_mass, com_x, com_y, com_z)
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

/// Sorts SoA bodies by morton code order in-place and returns
///  - array with the mapping from original to morton sorted indices
///  - array with the mapping from morton sorted to original indices
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
) -> (Vec<usize>, Vec<usize>, Vec<u64>) {
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

    let mut morton_to_original_idx = Vec::with_capacity(n);
    let mut sorted_morton_codes = Vec::with_capacity(n);
    for (code, idx) in morton_codes_and_idx.iter() {
        sorted_morton_codes.push(*code);
        morton_to_original_idx.push(*idx);

        // Print body and morton code during tests
        #[cfg(test)]
        {
            println!(
                "Body {}: original_idx={},\n\tmorton_code={:064b}",
                idx, idx, code,
            );
        }
    }

    let mut original_to_morton_idx = vec![0; n];
    for (morton_idx, &original_idx) in morton_to_original_idx.iter().enumerate() {
        original_to_morton_idx[original_idx] = morton_idx;
    }

    let mut scratch = Vec::with_capacity(n); // reused to avoid repeated vector creations
    let mut reorder = |data: &mut [f64]| {
        scratch.clear();
        for &i in &morton_to_original_idx {
            scratch.push(data[i]);
        }
        data.copy_from_slice(&scratch);
    };

    // Apply the same permutations to all SoA arrays
    reorder(masses);
    reorder(pos_x);
    reorder(pos_y);
    reorder(pos_z);

    (
        original_to_morton_idx,
        morton_to_original_idx,
        sorted_morton_codes,
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1_body_tree() {
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
    fn test_2_body_tree() {
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
    fn test_3_body_tree() {
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
