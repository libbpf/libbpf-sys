#ifdef __LIBBPF_SYS_NOVENDOR
#include <linux/if_link.h>
#include <linux/perf_event.h>
#include <bpf/bpf.h>
#include <bpf/btf.h>
#include <bpf/libbpf.h>
#else
#include "libbpf/include/uapi/linux/if_link.h"
#include "libbpf/include/uapi/linux/perf_event.h"
#include "libbpf/src/bpf.h"
#include "libbpf/src/btf.h"
#include "libbpf/src/libbpf.h"
#endif /* __LIBBPF_SYS_NOVENDOR */
