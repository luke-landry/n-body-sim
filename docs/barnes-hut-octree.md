# Barnes Hut Octrees

## Overview

An octree is a tree data structure where each node has 8 children, and can be used to represent regions of 3D space, where each node corresponds to a cubic region. The root node represents the entire cube of space, and each child node represents a subdivision of that space into 8 smaller cubes (octants).

In the context of the Barnes-Hut algorithm, an octree is used to efficiently compute gravitational forces between bodies. Each node in the octree contains information about the mass and center of mass (com) of the bodies contained within that influence of a group of bodies by using the information stored in a node, rather than having to compute the forces from each individual body, which can significantly reduce the computational complexity from $O(N^2)$ to $O(N\ log\ N)$ for $N$ bodies.

The nodes of a Barnes-Hut octree:
- **root node**: represents the entire space and contains the total mass and center of mass of all bodies.
- **internal nodes**: represent subdivisions of space and contain the mass and center of mass of the bodies within that subdivision
- **external (leaf) nodes**: represent individual bodies and contain the mass and position of that body, with no child nodes
## Implementation

### Data Structure

The intuitive way to implement an octree would be to use an AoS (Array of Structures) pointer-based structure, where the octree contains a set of node objects, and each node contains pointers to its 8 children. However, this can lead to inefficient memory access patterns and increased memory overhead due to the pointers. Using an SoA (Structure of Arrays) layout, where we store the properties of the nodes and bodies contiguously in separate arrays, can improve cache locality and reduce memory overhead, leading to better performance. It also means we can avoid dynamic memory (heap) allocation during octree construction and instead reuse pre-allocated arrays.

In order to preallocate the arrays for the octree nodes, we need to determine the maximum possible number of nodes that can be created based on the number of bodies (n) and the maximum number of bodies per leaf node (max_leaf_size).

Letting
    $N$ = number of bodies
    $L$ = number of leaf nodes
    $C$ = max number of bodies per leaf node
    $I$ = number of internal nodes
    $T$ = total number of nodes

Using a sparse tree, can assume assume each leaf node contains at least 1 body ($C \geq 1$)
    $L \lt N / C$

A node is subdivided into 8 children and becomes an internal node if it contains more than $C$ bodies, so internal nodes must have at least 1 body, which means
    $I \lt N$

Combining these inequalities gives us
    $T = L + I \lt N/C + N \lt 2N$

Thus, we can reach an upper bound for T
    $T \leq 2N - 1$

So, arrays storing node information can be preallocated with a capacity of $2N - 1$, and arrays storing body information can be preallocated with a capacity of $N$.

To efficiently organize the contiguous arrays of nodes/body information, we can use Morton codes. Morton codes, also known as Z-order curves, are a way to encode multi-dimensional data (like 3D coordinates) into a single dimension while preserving spatial locality. This means that points that are close together in 3D space will also be close together in the 1D Morton code space.
### Morton Codes

A Morton code takes the binary representation of positive integer (x, y, z) coordinates and interleaves their bits to create a single integer. For example, if we use 5 bits for each coordinate, we can take the bits of x, y, and z and interleave them from MSBs to LSBs to create a 15-bit Morton code in this pattern:
```
Morton code = z4 y4 x4 z3 y3 x3 z2 y2 x2 z1 y1 x1 z0 y0 x0
```

For example, if we have a coordinate $(x, y, z) = (5, 3, 6)$, we can represent these in binary as:
```
x = 5 (binary 00101)
y = 3 (binary 00011)
z = 6 (binary 00110)
```

Interleaving the bits of x, y, and z gives us the Morton code:
```
Morton code = z4 y4 x4 z3 y3 x3 z2 y2 x2 z1 y1 x1 z0 y0 x0
            = 0  0  0  0  0  0  1  0  1  1  1  0  0  1  1
            = binary 0000000101110011
            = decimal 371
```

Since we have 3 coordinates in 3D, the number of values that can be represented in a Morton code with n bits per coordinate is $2^{3n}$. For this 3D barnes-hut implementation, a reasonable choice to use is 21 bits per coordinate, which allows us to represent root_width/$2^{21}$ discrete positions in 3D space, which for a root_width of 20 units gives a spatial resolution of 0.0000095367 units. This means as long as the bodies are not closer than 0.0000095367 units apart, their positions can be distinguished in the octree. For example, if a unit is defined as 1 AU (astronomical unit), then 0.0000095367 units would be approximately 1427 kilometers, which should be a reasonable resolution for many astrophysical simulations.

Each set of three bits in a Morton code represents a level of the octree, with the values of these bits (b000 to b111) corresponding to the child node (0 to 7) of the node at that level. This means that there is no actual "node" entity being tracked. The Morton code of a body itself encodes its position (chain of parent nodes) that the body belongs to. This means a "node" is just a contiguous range of bodies in the sorted body arrays that share the same prefix of their Morton codes up to a certain bit level.

For example, in a simple case where we are using 3 bits per coordinate (so coordinates can be 0-7)
to give a 9 bit Morton code, and a body:
```
(x, y, z) = (4, 6, 1) = b(100, 110, 001) -> Morton code = 011 010 100
```

The nodes of the octree this body belongs to can be identified by the 3-bit groupings:
0. **level 0:** root node (since all bodies belong in the root node)
1. **level 1:** 5th child node of root node (since the first 3 bits of the morton code are b101 = 5)
2. **level 2:** 3rd child node of L1 node (since the next 3 bits of the morton code are b110 = 6)
3. **level 3:** 6th child node of L2 node (since the last 3 bits of the morton code are b011 = 3)

To convert f64 $(x, y, z)$ coordinates into the positive 21-bit integers that can be interleaved into
a Morton code, the f64 values need to be quantized into the range of $[0,\ 2^{21} - 1]$. This is done
by first normalizing the f64 values to the range $[0,\ 1]$ based on the minimum coordinate and width
of the root node, and then scaling this normalized value to the range of $[0,\ 2^{21} - 1]$.

### Building the Octree

Because of the Morton sorting, a "node" can be treated as a contiguous range of bodies in the sorted body arrays. e.g. a simple example if the octree had a bit_level starting at 3, with 16 bodies sorted in morton order and we build the octree, we might end up with something like this:
```
[b0,  b1,  b2,  b3,  b4,  b5,  b6,  b7,  b8,  b9,  b10, b11, b12, b13, b14, b15 ]
                                        |
                                        V
[m0,  m1,  m2,  m3,  m4,  m5,  m6,  m7,  m8,  m9,  m10, m11, m12, m13, m14, m15 ]
|AAAL|AABL|AACL|AADL|ABAL|ABBL|BAAL|BABL|BACL|BBAL|BBBL|BBCL|BBDL|CAAL|CBAL|CBBL| 
|-------AA----------|--AB-----|----BA--------|-----------BB------|-CA-|---CB----| 
|----------A------------------|-----------------B----------------|-------C------| 
|-----------------------------------root----------------------------------------| 
```

In this case, the (flat) node array would look like this:
```
[root,
A,
AA, AAAL, AABL, AACL, AADL,
AB, ABAL, ABBL,
B,
BA, BAAL, BABL, BACL,
BB, BBAL, BBBL, BBCL, BBDL,
C,
CA, CAAL
CB, CBAL, CBBL]
```
where each node stores the start index of its children in the node array (e.g. root would store 1 since its first child A is at index 1).

So, to track the nodes, we need to store the node's start index, and which of its children exist. Since the children $0..7$ of a node will be contiguous in the node arrays, we can just store the start index of the first child, and a bitmask indicating which children exist. From that information we can determine which children exist and what their offset from the start index is, and thus we can compute all child node indices.
