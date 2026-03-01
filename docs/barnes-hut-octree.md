# Barnes-Hut Octree

## Overview

An octree is a tree data structure where each node has 8 children, and can be used to represent regions of 3D space, where each node corresponds to a cubic region. The root node represents the entire cube of space, and each child node represents a subdivision of that space into 8 smaller cubes (octants).

In the the Barnes-Hut algorithm, an octree is used to efficiently compute gravitational forces between bodies. Each internal node in the octree contains information about the total mass and Center of Mass (CoM) of its children, and each leaf node contains information about the masses and positions bodies it contains, as well as their total mass and CoM. When a computing the gravitational forces on a target body, the algorithm traverses the octree, and for each node, it checks if the node is sufficiently far away from the target body to be approximated. How the algorithm determines if a node is far enough to approximate is based on the ratio of the size ($s$) of the node (the width of the cube it represents) to the distance ($d$) being less than a given threshold (denoted as $\theta$), given as $\frac{s}{d}<\theta$. If this condition is satisfied, the algorithm uses the total mass and CoM of that node to approximate the gravitational force from all the bodies contained within that node. Otherwise, the algorithm traverses each of the node's children nodes to compute the forces from those smaller subdivisions of space until the criterion is met or it reaches leaf nodes, in which case it directly computes the gravitational force of the target based on the masses and positions of bodies in the leaf nodes.

If $\theta=0$, the criterion will never be satisfied since $s>0$ and $d<\infty$, so for $N$ bodies, the algorithm will simply sum the forces individually which leads to a time complexity of $O(N^2)$. However, as the approximation threshold $\theta$ increases, the time complexity of calculating the forces on a target body approaches $O(N\ log\ N)$, which is much faster for large $N$, but reduces the accuracy of the force calculations as more of the forces are approximated.

The nodes of a Barnes-Hut octree:
- **root node**: represents the entire space and contains the total mass and center of mass of all bodies.
- **internal nodes**: represent subdivisions of space and contain the mass and center of mass of the bodies within that subdivision
- **external (leaf) nodes**: represent individual bodies and contain the mass and position of that body, with no child nodes

## Implementation

### Data Structure

The intuitive way to implement an octree would be to use an AoS (Array of Structures) pointer-based structure, where the octree contains a set of node objects, and each node contains pointers to its 8 children. However, this can lead to inefficient memory access patterns and increased memory overhead due to the pointers. Using an SoA (Structure of Arrays) layout, where we store the properties of the nodes and bodies contiguously in separate arrays, can improve cache locality and reduce memory overhead, leading to better performance. It also means we can avoid dynamic memory (heap) allocation during octree construction and instead reuse pre-allocated arrays.

In order to preallocate the arrays for the octree nodes, we need to determine the maximum possible number of nodes that can be created based on the number of bodies, the maximum number of bodies per leaf node, and the maximum depth of the tree.

Letting  
    $N$ = number of bodies  
    $L$ = number of leaf nodes  
    $C$ = max number of bodies per leaf node  
    $I$ = number of internal nodes  
    $D$ = maximum depth of the   
    $T$ = total number of nodes  

Using a sparse tree (empty leaves are not created), we can assume assume each leaf node contains at least 1 body ($C \geq 1$), so:  
    $L \lt \frac{N}{C}$

For a perfectly balanced tree, a node would be subdivided into 8 children and becomes an internal node if it contains more than $C$ bodies, so internal nodes must contain at least 1 body, which means:  
    $I \lt N$

However the tree would not be perfectly balanced unless the bodies are uniformly distributed in space, which is usually not the case. For example, in a situation where there are at least two bodies far apart, with one or both of these bodies having another body very close to it, the tree can become highly unbalanced, since the root node needs to cover the large distance between the two, and then this node has to be repeatedly subdivided to distinguish the bodies that are close together. A sun-earth-moon scenario is an example of this: the Earth and Moon are very close together compared to the distance between the Sun and Earth, so the octree's root covers the large distance between the Earth and Sun, but then would need to be subdivided many times so that the Earth and Moon lie in separate octants, leading to a large number of internal nodes. 

So, to handle a theoretical worst-case scenario where all leaves ($L$) reach the maximum possible depth of the tree ($D$), which means they each have $D$ parent internal nodes (including the root node), $I$ would be calculated as:  
  $I\lt LD$

Plugging these inequalities for $L$ and $I$ into the total of $T=L+I$:  
  $T=L+I\lt \frac{N}{C}+LD=\frac{N}{C}+\frac{N}{C}D=\frac{N}{C}(1+D)$

Thus, SoA arrays storing node information can be preallocated with a capacity of $\frac{N}{C}(1+D)$, and SoA arrays storing body information can be preallocated with a capacity of $N$. This allows the arrays to be reused across multiple time steps, and the octree can be rebuilt in-place at each time step by overwriting the node and body information in the arrays, instead of needing to dynamically allocate these arrays at each time step.

To efficiently organize the contiguous arrays of nodes/body information, we can use Morton codes. Morton codes, also known as Z-order curves, are a way to encode multi-dimensional data (like 3D coordinates) into a single dimension while preserving spatial locality. This means that points that are close together in 3D space will also be relatively close together in the 1D Morton code space.
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
1. **level 1:** 3rd child node of root node (since the first 3 bits of the morton code are b011 = 3)
2. **level 2:** 2nd child node of L1 node (since the next 3 bits of the morton code are b010 = 2)
3. **level 3:** 4th child node of L2 node (since the last 3 bits of the morton code are b100 = 4)

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

This data structure allows us to build the octree recursively in a depth-first manner, where we start with the root node (which corresponds to the entire range of bodies), and then for each node, we check if it contains more than $C$ bodies. If it does, we subdivide it into up to 8 children and then recursively build the octree for each child node. During the recursive construction, each node will compute it's own mass and CoM from the masses and CoMs of its children. For leaf nodes, mass and CoM is computed from the bodies it contains.

Because of the recursive structure, nodes will be constructed in this depth-first order:
```
[root,
A, AA, AAAL, AABL, AACL, AADL, AB, ABAL, ABBL,
B, BA, BAAL, BABL, BACL, BB, BBAL, BBBL, BBCL, BBDL,
C, CA, CAAL, CB, CBAL, CBBL]
```

To track a node's children indices, we can store the child node indices of each node in a vector of vectors, where the outer vector is indexed by the parent node index and the inner vector contains the child node indices.

### Traversing the Octree
To compute the gravitational forces on a target body, we can traverse the octree starting from the root node. For each node, we check if the node is sufficiently far away from the target body to be approximated using the criterion $\frac{s}{d}<\theta$. If this condition is satisfied, we use the total mass and CoM of that node to approximate the gravitational force from all the bodies contained within that node. Otherwise, we traverse each of the node's children nodes to compute the forces from those smaller subdivisions of space until the criterion is met or we reach leaf nodes, in which case we directly compute the gravitational force of the target based on the masses and positions of bodies in the leaf nodes.

This traversal can be implemented iteratively using a stack to avoid the performance overhead of recursion, which is more impactful for traversal (which is run N-times per step) compared to the recursive octree build which is only once per step. We push the root node onto the stack, and then in a loop, we pop a node from the stack, check the approximation criterion, and either compute the force or push its children onto the stack.
