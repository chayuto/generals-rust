/*
 * WP-7.2b forced-include prelude for the vendor EAC codec sources (pass with -include).
 *
 * gimex.h's ggetm()/gputm() take a native-endian shortcut when __APPLE__ is defined, written
 * for big-endian PowerPC Macs. On little-endian Apple Silicon this writes byte-swapped codec
 * headers. The system headers the codecs use are pulled in first, while __APPLE__ is still
 * defined; then __APPLE__ is removed, so gimex.h compiles its portable big-endian path, the
 * one Windows/Linux builds and retail data use.
 */
#include <stdlib.h>
#include <string.h>

#ifdef __APPLE__
#undef __APPLE__
#endif
