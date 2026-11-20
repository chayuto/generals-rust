/*
 * Exhaustive 2^32-input GameMath oracle digest (local Gate V2-FFI evidence for WP-2.2a).
 *
 * For a unary f32 function F, iterates every 32-bit pattern b, computes r = F(bits(b)),
 * canonicalizes NaN results to 0x7fc00000, and folds r into a 64-bit FNV-1a digest.
 * The Rust counterpart is `cargo run --release -p zh_common --example exhaustive_gamemath -- F`.
 *
 * Build: cc -O2 -ffp-contract=off -std=c99 \
 *          -Ivendor/GeneralsX/build/macos-vulkan/_deps/gamemath-src/include \
 *          tests/cpp_harness/exhaustive_gamemath.c \
 *          vendor/GeneralsX/build/macos-vulkan/_deps/gamemath-build/libgm.a -o /tmp/exhaustive_gamemath
 * Run:   /tmp/exhaustive_gamemath sinf
 */
#include <stdint.h>
#include <stdio.h>
#include <string.h>

#include "gmath.h"

typedef float (*unary_fn)(float);

static float f_sqrtf_abs(float x) { return gm_sqrtf(gm_fabsf(x)); }
static float f_atan2_y1(float x) { return gm_atan2f(1.0f, x); }
static float f_atan2_x1(float y) { return gm_atan2f(y, -1.0f); }

static unary_fn lookup(const char *name) {
    if (!strcmp(name, "sinf")) return gm_sinf;
    if (!strcmp(name, "cosf")) return gm_cosf;
    if (!strcmp(name, "tanf")) return gm_tanf;
    if (!strcmp(name, "asinf")) return gm_asinf;
    if (!strcmp(name, "acosf")) return gm_acosf;
    if (!strcmp(name, "atanf")) return gm_atanf;
    if (!strcmp(name, "sqrtf")) return f_sqrtf_abs;
    if (!strcmp(name, "floorf")) return gm_floorf;
    if (!strcmp(name, "ceilf")) return gm_ceilf;
    if (!strcmp(name, "rintf")) return gm_rintf;
    if (!strcmp(name, "roundf")) return gm_roundf;
    if (!strcmp(name, "atan2f_y1")) return f_atan2_y1;
    if (!strcmp(name, "atan2f_xm1")) return f_atan2_x1;
    return 0;
}

int main(int argc, char **argv) {
    if (argc != 2) { fprintf(stderr, "usage: %s <fn>\n", argv[0]); return 2; }
    unary_fn fn = lookup(argv[1]);
    if (!fn) { fprintf(stderr, "unknown fn %s\n", argv[1]); return 2; }
    uint64_t h = 0xcbf29ce484222325ull;
    uint32_t b = 0;
    do {
        float x, r;
        uint32_t rb;
        memcpy(&x, &b, 4);
        r = fn(x);
        memcpy(&rb, &r, 4);
        if ((rb & 0x7f800000u) == 0x7f800000u && (rb & 0x007fffffu) != 0) rb = 0x7fc00000u;
        h ^= rb;
        h *= 0x100000001b3ull;
        ++b;
    } while (b != 0);
    printf("%s %016llx\n", argv[1], (unsigned long long)h);
    return 0;
}
