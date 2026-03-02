#include <cstdint>
#include <cstdio>

extern "C" __global__ void gpu_init_check() {
  int i = threadIdx.x + blockIdx.x * blockDim.x;
  if (i == 0) {
    printf("GPU initialized\n");
  }
}

__device__ double3 compute_acceleration(double mass_j, double dx, double dy,
                                        double dz, double g_constant,
                                        double eps2) {
  double dist_sqr = dx * dx + dy * dy + dz * dz + eps2;
  double inv_dist = rsqrt(dist_sqr);
  double inv_dist3 = inv_dist * inv_dist * inv_dist;
  double k = g_constant * mass_j * inv_dist3;
  double ax = k * dx;
  double ay = k * dy;
  double az = k * dz;
  return make_double3(ax, ay, az);
}

extern "C" __global__ void newton_compute_accelerations(
    const double *masses, const double *pos_x, const double *pos_y,
    const double *pos_z, double *acc_x, double *acc_y, double *acc_z,
    uint32_t num_bodies, double g_constant, double eps2) {
  int i = threadIdx.x + blockIdx.x * blockDim.x;
  if (i >= num_bodies) {
    return;
  }

  double xi = pos_x[i];
  double yi = pos_y[i];
  double zi = pos_z[i];
  double ax = 0.0, ay = 0.0, az = 0.0;
  for (int j = 0; j < num_bodies; j++) {
    if (i != j) {
      double dx = pos_x[j] - xi;
      double dy = pos_y[j] - yi;
      double dz = pos_z[j] - zi;
      double3 acc =
          compute_acceleration(masses[j], dx, dy, dz, g_constant, eps2);
      ax += acc.x;
      ay += acc.y;
      az += acc.z;
    }
  }
  acc_x[i] = ax;
  acc_y[i] = ay;
  acc_z[i] = az;
}

extern "C" __global__ void euler_step(double *pos_x, double *pos_y,
                                      double *pos_z, double *vel_x,
                                      double *vel_y, double *vel_z,
                                      const double *acc_x, const double *acc_y,
                                      const double *acc_z, uint32_t num_bodies,
                                      double dt) {
  int i = threadIdx.x + blockIdx.x * blockDim.x;
  if (i >= num_bodies) {
    return;
  }

  // update velocity: v = v + a * dt
  double vx = vel_x[i] + acc_x[i] * dt;
  double vy = vel_y[i] + acc_y[i] * dt;
  double vz = vel_z[i] + acc_z[i] * dt;
  vel_x[i] = vx;
  vel_y[i] = vy;
  vel_z[i] = vz;

  // update position: p = p + v * dt
  pos_x[i] += vx * dt;
  pos_y[i] += vy * dt;
  pos_z[i] += vz * dt;
}
