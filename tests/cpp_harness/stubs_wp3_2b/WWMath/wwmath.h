#pragma once

struct WWMath {
    static inline float Ceilf(float x) { return gm_ceilf(x); }
    static inline float Floorf(float x) { return gm_floorf(x); }
    static inline float Sqrt(float x) { return gm_sqrtf(x); }
    static inline double Sqrt(double x) { return gm_sqrt(x); }
};
